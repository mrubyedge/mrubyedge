# mrubyedge-cli

[![crates.io](https://img.shields.io/crates/v/mrubyedge-cli.svg)](https://crates.io/crates/mrubyedge-cli)
[![docs.rs](https://docs.rs/mrubyedge-cli/badge.svg)](https://docs.rs/mrubyedge-cli)

Command-line interface for mruby/edge - a lightweight, WebAssembly-focused mruby VM implementation.

## About mruby/edge

mruby/edge is an mruby-compatible virtual machine implementation written in Rust, specifically designed for WebAssembly environments. It aims to provide:

- **WebAssembly-first design**: Optimized for running Ruby code in browsers and edge computing environments
- **Lightweight runtime**: Minimal footprint and binary size suitable for constrained environments
- **mruby compatibility**: Executes mruby bytecode (`.mrb` files) and Ruby source code
- **Rust safety**: Built with Rust for memory safety and reliability

## Installation

Install mrubyedge-cli using cargo:

```sh
cargo install mrubyedge-cli
```

Or build from source:

```sh
git clone https://github.com/mrubyedge/mrubyedge.git
cd mrubyedge
cargo build --release -p mrubyedge-cli
```

The binary will be available as `mrbedge`.

## Getting Started

Create a simple Ruby script `hello.rb`:

```ruby
puts "Hello from mruby/edge!"
puts RUBY_ENGINE
```

Run it with mrbedge:

```sh
mrbedge hello.rb
# or explicitly
mrbedge run hello.rb
```

## Main Features

### `run` - Execute Ruby Scripts

The `run` subcommand executes Ruby source files (`.rb`) or compiled mruby bytecode (`.mrb`).

**Usage:**
```sh
mrbedge run <file>
# or simply
mrbedge <file>
```

**Examples:**
```sh
# Run Ruby source
mrbedge run script.rb

# Run compiled bytecode
mrbedge run script.mrb
```

### `compile-mrb` - Compile Ruby to Bytecode

Compiles Ruby source code into mruby bytecode format for faster loading and distribution.

**Usage:**
```sh
mrbedge compile-mrb <input.rb> -o <output.mrb>
```

**Examples:**
```sh
# Compile a single file
mrbedge compile-mrb app.rb -o app.mrb

# Run the compiled bytecode
mrbedge run app.mrb
```

**Benefits:**
- Faster startup time (no parsing overhead)
- Smaller distribution size
- Protection of source code

### `wasm` - Generate WebAssembly Modules

Compiles Ruby code directly into a standalone WebAssembly module that can run in browsers or any WebAssembly runtime.

**Usage:**
```sh
mrbedge wasm <input.rb> -o <output.wasm>
```

**Examples:**
```sh
# Generate WebAssembly module
mrbedge wasm app.rb -o app.wasm

# Use in browser or Node.js
# The generated WASM can be loaded and executed in any WASM runtime
```

**Use Cases:**
- Serverless edge computing
- Browser-based applications
- Microservices with minimal overhead
- Cross-platform portable executables

#### WASI Support

The `wasm` command can generate both WASI-enabled and non-WASI WebAssembly binaries. By default, it produces WASI-enabled modules. To disable WASI support, use the `--no-wasi` flag.

- **WASI-enabled**: Supports file system access, environment variables, and standard I/O
- **Non-WASI**: Minimal pure WebAssembly suitable for browser environments with custom imports

#### Import/Export Functions

You can specify WebAssembly function imports and exports using RBS (Ruby Signature) files. Place RBS files with specific naming conventions alongside your Ruby script:

For a Ruby script named `foo.rb`:
- **`foo.import.rbs`**: Defines external functions to import from the WebAssembly host
- **`foo.export.rbs`**: Defines Ruby functions to export as WebAssembly functions

**Example:**

```ruby
# app.rb
def calculate(x, y)
  x + y
end
```

```rbs
# app.export.rbs
def calculate: (Integer, Integer) -> Integer
```

```rbs
# app.import.rbs
def external_log: (String) -> void
```

The generated WebAssembly module will expose `calculate` and can call `external_log` from the host environment.

> **Note**: Inline RBS annotations for imports and exports will be supported in future releases.

## Additional Resources

- [GitHub Repository](https://github.com/mrubyedge/mrubyedge)
- [API Documentation](https://docs.rs/mrubyedge-cli)
- [Core VM Documentation](https://docs.rs/mrubyedge)

## License

See the [LICENSE](../LICENSE) file in the repository root.
