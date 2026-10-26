# Version benchmarks

Run the same Ruby workloads and Rust harness against selected mruby/edge Git
revisions. Requires Python 3.11+, Git, Cargo/Rust (edition 2024), and the C build
tools required by `mruby-compiler2-sys`. Cargo needs registry access on the first
build. Running benchmarks needs no Python packages or extra benchmark tools;
the optional plotting tool below uses Matplotlib.

```sh
# Compare tags, branches, full/abbreviated commit SHAs, or remote-tracking refs.
# The first revision is the baseline. Omitting --ref selects HEAD.
python3 benchmarks/run.py --ref v2.1.2 --ref HEAD

# Include local, uncommitted Rust changes.
python3 benchmarks/run.py --ref HEAD --ref working-tree

# Select workloads and shorten the run for a smoke check.
python3 benchmarks/run.py --ref HEAD --case instance_variables --case hash \
  --samples 3 --sample-ms 20 --warmup-ms 50

# Measure only repeated execution in an initialized VM, with explicit features.
python3 benchmarks/run.py --ref v2.1.2 --ref HEAD --mode warm \
  --no-default-features --features wasi,mruby-hash-fnv --output /tmp/bench.json

python3 benchmarks/run.py --list
```

Cargo default features are enabled unless `--no-default-features` is specified.
In current releases these include `mrubyedge-debug`, whose per-instruction debug
environment checks affect performance even in a release build. To compare without
those checks, pass `--no-default-features --features wasi` to the whole comparison.

Refs must exist locally; fetch missing tags/branches first with `git fetch origin
--tags`. Each committed revision is extracted into an isolated directory using
`git archive`; your current branch and files are left in place. `working-tree`
uses the current source directory, including uncommitted edits. Avoid editing
Rust sources while builds are running.

The harness depends directly on that revision's `mrubyedge/` directory. It does
not use the CLI, which can depend on a separately published VM version. Each
version gets a release executable, a standalone manifest, a Cargo.lock and a build
log. Compilation artifacts are reused under `target/benchmarks/build/`; source
snapshots and run artifacts live in a fresh `target/benchmarks/run-*/` directory.
Builds finish before timing begins. This requires no checkout, worktree, or
changes to historical commits.

The compiler version is inferred from each revision's `mruby-compiler2-sys`
dev-dependency and pinned to its declared release, so both mruby 3.x and 4.x
bytecode formats can be tested. Use `--compiler-version 0.5.0` to override it.
Old revisions without that dependency need an explicit compatible compiler.
Revisions must expose `rite::load`, `VM::open`, `VM::run`, `mrb_funcall`, and the
integer result conversion used in `runner.rs`; incompatible APIs, unsupported
Ruby behavior, or build failures stop the run with a nonzero exit status.
Version-specific bytecode/compiler changes are part of the comparison; this
does not force identical bytecode across incompatible VM versions.

## What is measured

This suite measures the **native host VM**. It does not measure WebAssembly,
WASI runtime overhead, CLI startup, or Ruby compilation speed.

Each workload defines `benchmark`, returns a deterministic integer, and creates
its own data on every invocation. The runner checks the result on every iteration
and fails on errors or mismatches. Ruby compilation happens once, before timing.

| Mode | Included in one timed iteration |
| --- | --- |
| `cold` | Bytecode loading, VM creation, top-level class/method definitions, one `benchmark` call, result check, destruction |
| `warm` | One `benchmark` call in an initialized VM, result check and result destruction |

`--mode both` (default) measures both separately. Each case/mode/version runs in a
separate process. Native calls into Ruby and workload object allocations are
included. Warm mode reuses the VM and its caches, but the workload resets its
own data. Cold mode includes initialization, so its result is not a pure execution
time. There is no process startup or build time in either measurement.

The runner warms up for 300 ms, calibrates an iteration count, then collects 15
batches targeting 100 ms each. An iteration is a **whole workload invocation**,
not a single Ruby operation. Very slow invocations can exceed the requested batch
duration; very fast ones are capped at 1,000,000 iterations per batch. Warmup and
calibration are discarded. Median, mean, minimum and sample standard deviation
use the elapsed time per iteration in each batch.

The Markdown comparison reports median milliseconds per invocation. `vs first`
is `baseline median / selected median`: **above 1.0x is faster**. These descriptive
ratios are not significance tests. Run on an idle machine with the same Rust
toolchain and feature flags, repeat comparisons, and reverse revision order to
check thermal/order effects before drawing conclusions.

## Results and reproduction

The default output is `results.json` plus `results.md` inside the run directory.
`--output PATH` selects another JSON path and writes Markdown beside it. JSON
records commit SHAs, working-tree dirty status, crate and compiler versions,
feature flags, Rust/Cargo versions, platform, environment flags, executable sizes,
runner/workload hashes, lockfile hashes, raw batch iteration counts and durations,
and summary statistics. Partial JSON is saved after each successful measurement;
a failed run exits nonzero and must not be treated as a completed comparison.
Only a report with `status: "complete"` represents a completed run.

Keep the entire run directory to retain source snapshots, workload files,
generated harnesses, lockfiles, executables, raw TSV samples and logs. Cargo's
standalone lockfiles preserve the actual dependency resolution for each build;
the historical workspace lockfile is not applied to the generated project.
For `working-tree`, record or commit the source separately: a dirty flag and HEAD
SHA do not capture uncommitted changes. The binary size is for the complete
native harness including the Ruby compiler; it is not the VM or Wasm size.

Results and build artifacts are ignored by Git under `target/`. Remove
`target/benchmarks/` when no longer needed.

## Manual GitHub Actions runs

After this workflow is merged into the default branch, open **Actions → Native
VM benchmark → Run workflow**. The branch selector chooses the benchmark tools
and Ruby workloads. The separate `ref` input chooses the VM source to build;
selecting an old VM ref does not replace the benchmark tools with old files.

| Input | Default | Meaning |
| --- | --- | --- |
| `ref` | `HEAD` | Target commit SHA, tag/version or branch |
| `baseline_ref` | `v1.1.12` | First version for comparison; empty measures only `ref` |
| `cases` | empty | Comma-separated workload names; empty runs all |
| `mode` | `both` | `cold`, `warm` or `both` |
| `samples` | `15` | Samples per workload/mode, from 1 to 100 |
| `features` | `wasi` | Comma-separated VM features |
| `default_features` | `false` | Enable Cargo defaults, including debug checks |

`HEAD` means the exact commit selected by the workflow's branch selector at
dispatch time. An explicit SHA stays independent of that branch. Version numbers
such as `1.1.12` also resolve to `v1.1.12`; branch names resolve locally or through
`origin/`. All Git history and tags are fetched. `working-tree` is not accepted
in CI. Both revisions use the same runner, workloads, Rust 1.97.1 toolchain,
feature settings and Ubuntu 24.04 runner, sequentially in one job.

For example, using GitHub CLI after merging:

```sh
gh workflow run benchmark.yml --ref master \
  -f ref=621256668d372d695733d482ec402eb3f27ce23c \
  -f baseline_ref=v1.1.12 -f cases=cart,html_render,request_routing
```

The **benchmark** job summary contains the table and an Artifact download link.
The **publish-charts** summary displays the table and embedded time/speedup PNGs.
JSON, Markdown and PNG files are uploaded as `benchmark-report`; build logs, raw
samples and lockfiles are uploaded as `benchmark-logs`, including on failures.
Artifacts are retained for 30 days.

To make chart images directly viewable in the summary, the publication job
creates or updates a separate `benchmark-results` branch, storing reports under
`runs/<run-id>-<attempt>/`. Summaries use image URLs pinned to the publication
commit; older URLs may expire after snapshot retention and GitHub garbage collection. The branch keeps snapshots of the latest 30 runs independently of Artifact expiration. This
publication assumes a public GitHub repository, so image URLs are readable by
GitHub's image proxy. No Pages setup or deployment secret is needed. Only the
publication job has `contents: write`; the build job has read-only permissions
and does not retain checkout credentials. Repository policies must permit the
publication job to push to `benchmark-results`.

Hosted runner load and hardware can vary between runs. Compare revisions within
one run and repeat runs before treating small differences as regressions.

## Plotting JSON results

Install the optional plotting dependency into a virtual environment once:

```sh
python3 -m venv target/benchmarks/plot-venv
target/benchmarks/plot-venv/bin/python -m pip install -r benchmarks/requirements-plot.txt
```

Pass a completed result JSON to generate a PNG beside it:

```sh
target/benchmarks/plot-venv/bin/python benchmarks/plot.py path/to/results.json

# Compare median speedups against the first version, with 1x as the reference.
target/benchmarks/plot-venv/bin/python benchmarks/plot.py path/to/results.json \
  --metric speedup --output /tmp/speedup.svg

# Select workloads and a measurement mode; PNG, SVG and PDF are supported.
target/benchmarks/plot-venv/bin/python benchmarks/plot.py path/to/results.json \
  --case cart --case html_render --mode warm --output /tmp/application.png
```

Default output names are `results-time.png` and `results-speedup.png`. Each
workload/mode has its own panel and linear scale; all versions remain visible.
Time charts show median milliseconds per whole invocation (lower is faster),
with whiskers for the observed min/max of raw batch times per iteration. These
are not confidence intervals. Speedup charts show the first version's median
divided by each version's median (higher is faster); they do not show uncertainty
or establish statistical significance. SHA labels are shortened for readability.

The tool works offline after dependency installation and does not rerun benchmarks.
It accepts only complete schema-version-1 reports and rejects missing selected
measurements, unsupported units and invalid timing data. Single-version and
single-mode reports are supported. The standard Python tests below do not require
Matplotlib; only rendering needs it.

## Workloads

| Name | Operations |
| --- | --- |
| `integer` | Integer arithmetic and a while loop |
| `method_calls` | Ruby method dispatch and counter updates |
| `instance_variables` | Multiple ivar slots, direct access and attr accessors |
| `hash` | String-key inserts and lookups with interpolation |
| `array` | Array growth, block calls and iteration |
| `string` | String concatenation, slicing and length |
| `cart` | Build carts with 100, 10 and 2 line items; compute category-dependent member discounts, coupons, tax and shipping |
| `html_render` | Build 100 article records; escape titles/authors and render 75 published articles into an HTML list |
| `request_routing` | Build and dispatch 300 requests through method/path lookup, authorization, controllers and response construction |

The application workloads include input generation and result validation in each
invocation, in both modes. `html_render` also computes a rolling checksum over the
complete output inside the timed region. `request_routing` covers successful
product, order and account requests, unauthorized requests, missing paths and
unsupported methods; it performs no network I/O. Their results describe the
whole workflow, including allocation and validation, rather than an isolated
rendering, pricing or dispatch operation.
These workloads use `while` loops to avoid historical block/upvar handling bugs
in v1.1.12 when blocks call into other Ruby methods.

Add a Ruby file under `workloads/` with a first line `# expected: INTEGER` and a
zero-argument `benchmark` method returning that integer. Avoid output, randomness,
external I/O, or persistent state that changes between calls. The runner lists
and discovers files automatically; `--case NAME` selects the filename stem.

Run orchestration checks with:

```sh
python3 -m unittest discover -s benchmarks -p 'test_*.py'
```

## Workflow security

Manual CI runs use two restricted Docker containers built from the same
SHA-256-pinned Rust 1.97.1 image. The first only resolves/fetches dependencies;
it has network access and never compiles or executes the selected revision.
The second builds and measures with `--locked --offline` and Docker network
mode `none`. No host Actions variables, tokens, Docker socket or home directory
are passed into either container. Both run as the runner's non-root UID, drop
all capabilities, forbid privilege escalation, and use a read-only root and
source mount. Only `target/`, a fresh Cargo cache and a private 1 GiB `/tmp`
are writable. Limits are 2 CPUs, 6 GiB RAM, 256 processes, 128 MiB per file and
20 minutes per container; builds and individual workloads have shorter limits.
Docker is an isolation boundary, not a separate kernel. Do not use a persistent
self-hosted runner for untrusted revisions. Debian packages are installed from
signed repositories when building the pinned base; rebuilds may include OS
security updates and are not bit-for-bit reproducible.

The runner checks expected revision SHAs, dependency lock hashes, workload and
runner hashes, sample counts, ranges and recalculated statistics outside the
containers. This expectation is recorded before the selected code runs and is
not writable inside the containers. Sanitized JSON is the only plotting input.
The producer's Markdown is discarded and regenerated. Publication checks the
same metadata again, rejects extra files, symlinks, special/oversized files,
and checks PNG checksums, dimensions and bounded decompression. This prevents
format/path attacks; a malicious VM can still fabricate plausible timings.

Only the protected `master` workflow can publish, and all selected dependency
configurations must have [reviewed harness locks](locks/README.md). Other runs
still produce downloadable reports. The `benchmark-publication` Environment
must allow only the `master` branch. The publication checkout does not retain
credentials; the write token is supplied only to the final push step. External
Actions in that job still inherit job-level token permissions, so SHA pinning
and review remain necessary. The result branch is never checked out: validated
blobs are added with Git plumbing in an empty temporary repository.

Publication keeps the newest 30 runs in a parentless snapshot. Updating only
`benchmark-results` uses an explicit force-with-lease against the fetched SHA,
so a competing push causes failure rather than data loss. Reachable history
is bounded; older image URLs may eventually stop working after GitHub garbage
collection. Reports in the downloadable artifacts expire after 30 days.
Diagnostic artifacts contain regular files copied to a separate directory,
with per-file/count/total limits; raw container output is never streamed as
Actions workflow commands or traversed directly by upload-artifact.

Python direct and transitive versions and distribution hashes are locked in
`requirements-plot.txt`; CI only accepts wheels. To update the lock in a clean
Python 3.11+ environment with pip-tools installed:

```sh
pip-compile --generate-hashes --strip-extras --no-emit-index-url \
  --no-emit-trusted-host --output-file benchmarks/requirements-plot.txt \
  benchmarks/requirements-plot.in
python3 benchmarks/audit_dependencies.py
```

Review the version and hash changes before merging. Dependabot proposes
Python, Action and container updates. GitHub Actions are pinned throughout
this repository so repository-wide enforcement can be enabled safely after
merging this change. Repository controls are previewed/applied with:

```sh
python3 scripts/configure_actions_security.py
python3 scripts/configure_actions_security.py --apply
# Run only AFTER all pinned workflows have reached master. The script refuses
# to enable SHA enforcement while any default-branch Action is still unpinned.
python3 scripts/configure_actions_security.py --apply --enable-sha-pinning
```

The controls require a reviewed PR and resolved discussions for `master`, block
force pushes/deletion, protect version tags from updates/deletion, and restrict
the publication Environment. Repository admins may bypass PR review only via
a PR (for solo-maintainer operation); Actions have no bypass entry. Admin tag
bypass remains available for deliberate release repair. CODEOWNERS requests
review for workflow, benchmark and dependency-manifest changes.
