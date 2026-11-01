#!/usr/bin/env python3
"""Translate workflow inputs into safe run.py arguments; also usable locally."""
import argparse
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def resolve_ref(value):
    value = value.strip()
    if not value or value == "working-tree":
        raise ValueError("ref must be a committed Git ref, SHA, version or HEAD")
    candidates = [value]
    if re.fullmatch(r"\d+\.\d+\.\d+(?:[-+][\w.-]+)?", value):
        candidates.append("v" + value)
    candidates.append("origin/" + value)
    for candidate in candidates:
        result = subprocess.run(
            ["git", "rev-parse", "--verify", "--end-of-options", candidate + "^{commit}"],
            cwd=ROOT, capture_output=True, text=True)
        if result.returncode == 0:
            commit = result.stdout.strip()
            if re.fullmatch(r"[0-9a-f]{40}", commit):
                return commit
    raise ValueError(f"cannot resolve benchmark ref: {value}")


def arguments(inputs, output):
    for name in ("BENCHMARK_REF", "BASELINE_REF", "BENCHMARK_FEATURES", "BENCHMARK_CASES"):
        if len(inputs.get(name, "")) > 1024:
            raise ValueError(f"{name} is too long")
    features = inputs.get("BENCHMARK_FEATURES", "wasi")
    if features and not re.fullmatch(r"[A-Za-z0-9_, /.-]+", features):
        raise ValueError("invalid feature names")
    target = resolve_ref(inputs.get("BENCHMARK_REF", "HEAD"))
    baseline = inputs.get("BASELINE_REF", "v1.1.12").strip()
    samples = int(inputs.get("BENCHMARK_SAMPLES", "15"))
    if not 1 <= samples <= 100:
        raise ValueError("samples must be between 1 and 100")
    mode = inputs.get("BENCHMARK_MODE", "both")
    if mode not in ("both", "cold", "warm"):
        raise ValueError("unknown measurement mode")
    defaults = inputs.get("DEFAULT_FEATURES", "false").lower()
    if defaults not in ("true", "false"):
        raise ValueError("default_features must be true or false")
    args = [sys.executable, str(ROOT / "benchmarks/run.py")]
    if baseline:
        args.extend(["--ref", resolve_ref(baseline)])
    args.extend(["--ref", target, "--samples", str(samples), "--mode", mode,
                 "--features=" + inputs.get("BENCHMARK_FEATURES", "wasi"),
                 "--output", str(output)])
    if defaults == "false":
        args.append("--no-default-features")
    for name in inputs.get("BENCHMARK_CASES", "").split(","):
        if name.strip():
            args.append("--case=" + name.strip())
    return args


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    try:
        subprocess.run(arguments(os.environ, options.output), cwd=ROOT, check=True)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"CI benchmark failed: {error}\n")


if __name__ == "__main__":
    main()
