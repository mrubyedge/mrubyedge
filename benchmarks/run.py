#!/usr/bin/env python3
"""Build the same native benchmark harness against arbitrary local Git revisions."""
import argparse
from datetime import datetime, timezone
import hashlib
import html
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import sys
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
SUITE = ROOT / "benchmarks"


def command(*args, cwd=None, env=None):
    return subprocess.check_output(args, cwd=cwd or ROOT, env=env, text=True).strip()


def positive(value):
    number = int(value)
    if number <= 0:
        raise argparse.ArgumentTypeError("must be a positive integer")
    return number


def nonnegative(value):
    number = int(value)
    if number < 0:
        raise argparse.ArgumentTypeError("must be a nonnegative integer")
    return number


def summarize(rows, samples):
    parsed = [tuple(map(int, row.split("\t"))) for row in rows.splitlines()]
    if len(parsed) != samples or any(
        len(row) != 3 or row[0] != index or row[1] <= 0 or row[2] <= 0
        for index, row in enumerate(parsed)
    ):
        raise ValueError("invalid benchmark output (workloads must not print to stdout)")
    timings = [elapsed / iterations for _, iterations, elapsed in parsed]
    return {
        "unit": "ns/iteration",
        "median": statistics.median(timings),
        "mean": statistics.mean(timings),
        "min": min(timings),
        "stdev": statistics.stdev(timings) if len(timings) > 1 else 0,
        "samples": [
            {"iterations": iterations, "elapsed_ns": elapsed}
            for _, iterations, elapsed in parsed
        ],
    }


def compiler_requirement(manifest, override=None):
    compiler = override or manifest.get("dev-dependencies", {}).get("mruby-compiler2-sys")
    if isinstance(compiler, dict):
        compiler = compiler.get("version")
    if not isinstance(compiler, str) or not compiler:
        raise ValueError("cannot detect mruby-compiler2-sys; use --compiler-version")
    return "=" + compiler if compiler[0].isdigit() else compiler


def dependency_key(manifest, options):
    # Source-only changes can reuse the reviewed resolution. Cargo --locked also
    # checks the full dependency graph, including any path dependencies.
    specification = {"manifest": manifest,
                     "compiler": compiler_requirement(manifest, options.compiler_version),
                     "features": sorted(options.features),
                     "default_features": not options.no_default_features}
    return hashlib.sha256(json.dumps(specification, sort_keys=True).encode()).hexdigest()


def dependency_lock(version, project, options, env):
    key = version["dependency_key"]
    reviewed = SUITE / "locks" / (key + ".lock")
    resolved = ROOT / "target/benchmarks/resolved-locks" / (key + ".lock")
    lock = project / "Cargo.lock"
    if options.lock_policy != "refresh" and reviewed.is_file():
        shutil.copyfile(reviewed, lock)
        version["dependencies_reviewed"] = True
    elif options.lock_policy == "require":
        raise ValueError(f"no reviewed lockfile for {key}")
    elif options.offline:
        shutil.copyfile(resolved, lock)
        version["dependencies_reviewed"] = False
    else:
        subprocess.run(["cargo", "generate-lockfile", "--manifest-path", str(project / "Cargo.toml")],
                       cwd=project, env=env, check=True, timeout=180)
        version["dependencies_reviewed"] = False
    if options.prepare_only:
        subprocess.run(["cargo", "fetch", "--locked", "--manifest-path", str(project / "Cargo.toml")],
                       cwd=project, env=env, check=True, timeout=300)
        resolved.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(lock, resolved)
    version["lock_sha256"] = hashlib.sha256(lock.read_bytes()).hexdigest()


def prepare(ref, directory, options, runner_source=None):
    commit = command("git", "rev-parse", "--verify", "--end-of-options",
                     ("HEAD" if ref == "working-tree" else ref) + "^{commit}")
    if ref == "working-tree":
        source = ROOT
        dirty = bool(command("git", "status", "--porcelain"))
    else:
        source = directory / "source"
        source.mkdir()
        archive = directory / "source.tar"
        with archive.open("wb") as output:
            subprocess.run(["git", "archive", "--format=tar", commit], cwd=ROOT,
                           stdout=output, check=True)
        with tarfile.open(archive) as contents:
            contents.extractall(source, filter="data")
        archive.unlink()
        dirty = False
    crate = source / "mrubyedge"
    manifest = tomllib.loads((crate / "Cargo.toml").read_text())
    compiler = compiler_requirement(manifest, options.compiler_version)
    project = directory / "runner"
    (project / "src").mkdir(parents=True)
    shutil.copyfile(runner_source or SUITE / "runner.rs", project / "src/main.rs")
    (project / "Cargo.toml").write_text(
        '[package]\nname = "mrubyedge-version-benchmark"\nversion = "0.0.0"\n'
        'edition = "2024"\npublish = false\n\n[workspace]\n\n[dependencies]\n'
        f'mrubyedge = {{ path = {json.dumps(str(crate))}, '
        f'default-features = {str(not options.no_default_features).lower()}, '
        f'features = {json.dumps(options.features)} }}\n'
        f'mruby-compiler2-sys = {json.dumps(compiler)}\n'
    )
    return {"ref": ref, "commit": commit, "dirty": dirty,
            "crate_version": manifest["package"]["version"], "compiler": compiler,
            "project": str(project),
            "dependency_key": dependency_key(manifest, options)}


def table(report):
    baseline = report["versions"][0]
    lines = ["| Version | Workload | Mode | Median (ms) | vs first |",
             "| --- | --- | --- | ---: | ---: |"]
    for version in report["versions"]:
        for key, result in version["results"].items():
            base = baseline["results"].get(key)
            ratio = f'{base["median"] / result["median"]:.3f}x' if base else "—"
            name, mode = key.split("/")
            label = html.escape(version["ref"], quote=True).replace("|", "&#124;").replace("\n", " ").replace("\r", " ")
            label = label.replace("[", "&#91;").replace("]", "&#93;").replace("`", "&#96;").replace("!", "&#33;").replace("\\", "&#92;")
            lines.append(f'| {label} | {name} | {mode} | '
                         f'{result["median"] / 1e6:.3f} | {ratio} |')
    return "\n".join(lines)


def parse_options(args=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ref", action="append", help="Git ref, tag, SHA or working-tree; repeat to compare (default: HEAD)")
    parser.add_argument("--case", action="append", help="workload name; repeat to select multiple")
    parser.add_argument("--mode", choices=["cold", "warm", "both"], default="both")
    parser.add_argument("--samples", type=positive, default=15)
    parser.add_argument("--sample-ms", type=positive, default=100)
    parser.add_argument("--warmup-ms", type=nonnegative, default=300)
    parser.add_argument("--features", default="", help="comma-separated mrubyedge features")
    parser.add_argument("--no-default-features", action="store_true")
    parser.add_argument("--compiler-version", help="override the compiler version inferred from the selected source")
    parser.add_argument("--output", type=Path, help="JSON result path (default: run directory/results.json)")
    parser.add_argument("--list", action="store_true", help="list workloads without building")
    parser.add_argument("--prepare-only", action="store_true", help="fetch dependencies without compiling or executing source")
    parser.add_argument("--offline", action="store_true", help="use fetched dependencies without network access")
    parser.add_argument("--lock-policy", choices=["prefer", "require", "refresh"], default="prefer")
    options = parser.parse_args(args)
    if options.prepare_only and options.offline:
        parser.error("--prepare-only and --offline are mutually exclusive")
    if options.output and options.output.suffix.lower() != ".json":
        parser.error("--output must have a .json extension")
    options.features = [f.strip() for f in options.features.split(",") if f.strip()]
    return options


def main():
    options = parse_options()
    available = {p.stem: p for p in sorted((SUITE / "workloads").glob("*.rb"))}
    if options.list:
        print("\n".join(available))
        return
    cases = list(dict.fromkeys(options.case or available))
    unknown = set(cases) - available.keys()
    if unknown:
        raise ValueError("unknown workloads: " + ", ".join(sorted(unknown)))
    modes = ["cold", "warm"] if options.mode == "both" else [options.mode]
    cache = ROOT / "target/benchmarks"
    cache.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="run-", dir=cache))
    print(f"Artifacts: {directory}", flush=True)
    report = {
        "schema_version": 1,
        "status": "running",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "environment": {"platform": platform.platform(), "machine": platform.machine(),
                        "processor": platform.processor(), "cpu_count": os.cpu_count(),
                        "rustc": command("rustc", "-vV"), "cargo": command("cargo", "--version"),
                        "RUSTFLAGS": os.environ.get("RUSTFLAGS", ""),
                        "CARGO_ENCODED_RUSTFLAGS": os.environ.get("CARGO_ENCODED_RUSTFLAGS", ""),
                        "MRUBYEDGE_DEBUG": os.environ.get("MRUBYEDGE_DEBUG"),
                        "RUSTUP_TOOLCHAIN": os.environ.get("RUSTUP_TOOLCHAIN", "")},
        "config": {"samples": options.samples, "sample_ms": options.sample_ms,
                   "warmup_ms": options.warmup_ms, "features": options.features,
                   "default_features": not options.no_default_features,
                   "profile": "release", "modes": modes},
        "runner_sha256": hashlib.sha256((SUITE / "runner.rs").read_bytes()).hexdigest(),
        "workloads": {}, "versions": [],
    }
    output = (options.output or directory / "results.json").resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + "\n")
    # Freeze inputs for the whole comparison, and retain them for reproduction.
    runner_source = directory / "runner.rs"
    shutil.copyfile(SUITE / "runner.rs", runner_source)
    workloads = directory / "workloads"
    workloads.mkdir()
    for name in cases:
        data = available[name].read_bytes()
        first = data.decode().splitlines()[0]
        if not first.startswith("# expected: "):
            raise ValueError(f"{name}: missing '# expected: INTEGER' header")
        expected = int(first.removeprefix("# expected: "))
        (workloads / f"{name}.rb").write_bytes(data)
        report["workloads"][name] = {"expected": expected, "sha256": hashlib.sha256(data).hexdigest()}
    host = next(line.removeprefix("host: ") for line in report["environment"]["rustc"].splitlines()
                if line.startswith("host: "))
    report["config"]["target"] = host
    env = dict(os.environ, CARGO_TARGET_DIR=str(cache / "build"))
    # Finish all builds before measuring any version.
    for index, ref in enumerate(options.ref or ["HEAD"]):
        version_dir = directory / str(index)
        version_dir.mkdir()
        print(f"Building {ref} (release)...", flush=True)
        version = prepare(ref, version_dir, options, runner_source)
        project = Path(version["project"])
        dependency_lock(version, project, options, env)
        if options.prepare_only:
            report["versions"].append(version)
            continue
        with (version_dir / "build.log").open("w") as log:
            subprocess.run(["cargo", "build", "--locked", *(["--offline"] if options.offline else []), "--release", "--target", host,
                            "--manifest-path", str(project / "Cargo.toml")],
                           cwd=project, env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)
        executable = version_dir / ("bench-runner.exe" if os.name == "nt" else "bench-runner")
        shutil.copy2(cache / "build" / host / "release" /
                     ("mrubyedge-version-benchmark.exe" if os.name == "nt"
                      else "mrubyedge-version-benchmark"), executable)
        version.update(executable=str(executable), binary_bytes=executable.stat().st_size,
                       lock_sha256=hashlib.sha256((project / "Cargo.lock").read_bytes()).hexdigest(), results={})
        report["versions"].append(version)
        output.write_text(json.dumps(report, indent=2) + "\n")
    if options.prepare_only:
        report["status"] = "dependencies-ready"
        output.write_text(json.dumps(report, indent=2) + "\n")
        return
    for version in report["versions"]:
        for name in cases:
            for mode in modes:
                print(f'Measuring {version["ref"]}: {name}/{mode}...', flush=True)
                prefix = Path(version["executable"]).parent / f"{name}-{mode}"
                with prefix.with_suffix(".stderr.log").open("w") as log, prefix.with_suffix(".tsv").open("w+") as samples:
                    subprocess.run(
                        [version["executable"], str(workloads / f"{name}.rb"), mode,
                         str(options.samples), str(options.sample_ms), str(options.warmup_ms),
                         str(report["workloads"][name]["expected"])],
                        cwd=directory, env=env, stdout=samples, stderr=log, text=True, check=True, timeout=120)
                    if samples.tell() > 1024 * 1024:
                        raise ValueError("benchmark output exceeds 1 MiB")
                    samples.seek(0)
                    rows = samples.read(1024 * 1024 + 1)
                version["results"][f"{name}/{mode}"] = summarize(rows, options.samples)
                output.write_text(json.dumps(report, indent=2) + "\n")
    report["status"] = "complete"
    output.write_text(json.dumps(report, indent=2) + "\n")
    rendered = table(report)
    output.with_suffix(".md").write_text(rendered + "\n")
    print("\n" + rendered + f"\n\nResults: {output}")


if __name__ == "__main__":
    try:
        main()
    except (subprocess.SubprocessError, OSError, ValueError) as error:
        print(f"Benchmark failed: {error}\nSee the artifact directory for build and workload logs.", file=sys.stderr)
        sys.exit(1)
