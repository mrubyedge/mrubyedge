# Reviewed benchmark dependency resolutions

Each `<key>.lock` is the complete lockfile of the generated standalone harness.
The key hashes the selected VM manifest, compiler requirement, features and
Cargo default-feature setting. Code-only VM changes can reuse a resolution;
Cargo `--locked` rejects incompatible dependency changes, including changes in
path dependencies. These locks are independent of the root workspace lockfile.

Initial resolutions cover `v1.1.12` and VM version `2.1.2`, with defaults disabled
and `features=wasi`. All external packages use crates.io checksums. The locks
were checked against OSV; the security workflow repeats this check for updates.
Absence of an advisory is not a source-code audit or evidence against malware.

To review a new configuration, use an isolated manual benchmark run. Download
its bounded logs artifact (which includes the generated Cargo.lock), inspect
all dependency/source/version/checksum changes and advisories, then add that
lock as `<dependency_key>.lock`. The key is recorded in the report. To generate
one locally without compiling any selected source:

```sh
python3 benchmarks/run.py --ref REVISION --no-default-features --features wasi \
  --prepare-only --lock-policy refresh
```

The resulting file is in `target/benchmarks/resolved-locks/`. Never promote a
lockfile merely because the build completed. Commit reviewed updates through
normal review; subsequent CI builds fetch with `--locked`, then compile with
`--locked --offline` in a network-disabled container. Unknown configurations
remain measurable and downloadable but cannot publish automatically.
