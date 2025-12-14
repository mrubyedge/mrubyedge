# mrubyedge

[![crates.io](https://img.shields.io/crates/v/mrubyedge.svg)](https://crates.io/crates/mrubyedge)
[![docs.rs](https://docs.rs/mrubyedge/badge.svg)](https://docs.rs/mrubyedge)

A pure-Rust reimplementation of the mruby VM that keeps its core execution engine `no_std`-friendly while striving for behavioral compatibility with upstream mruby.

## Overview

mruby/edge is an mruby-compatible virtual machine implementation written in Rust, specifically designed for WebAssembly environments and embedded systems. It aims to provide:

- **WebAssembly-first design**: Optimized for running Ruby code in browsers and edge computing environments
- **Lightweight runtime**: Minimal footprint and binary size suitable for constrained environments
- **`no_std` core**: Can run in environments without standard library support
- **mruby compatibility**: Executes mruby bytecode (`.mrb` files) and Ruby source code
- **Rust safety**: Built with Rust for memory safety and reliability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mrubyedge = "1.0"
```

## Usage

### Running Precompiled Bytecode

Load and execute a precompiled `*.mrb` file produced by `mrbc`:

```rust
use mrubyedge::rite;
use mrubyedge::yamrb::vm;

// Bundle the compiled script at build time
const SCRIPT: &[u8] = include_bytes!("./examples/simple.mrb");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rite = rite::load(SCRIPT)?;
    let mut vm = vm::VM::open(&mut rite);
    let value = vm.run()?;
    println!("{:?}", value);
    Ok(())
}
```

### Creating VMs Programmatically

You can also construct IREP (internal representation) structures directly:

```rust
use mrubyedge::yamrb::{op, vm, value::RSym};
use mrubyedge::rite::insn::{Fetched, OpCode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let irep = vm::IREP {
        __id: 0,
        nlocals: 0,
        nregs: 7,
        rlen: 0,
        code: vec![
            op::Op { code: OpCode::LOADI_1, operand: Fetched::B(1), pos: 0, len: 2 },
            op::Op { code: OpCode::LOADI_2, operand: Fetched::B(2), pos: 2, len: 2 },
            op::Op { code: OpCode::ADD, operand: Fetched::B(1), pos: 4, len: 2 },
            op::Op { code: OpCode::STOP, operand: Fetched::Z, pos: 6, len: 1 },
        ],
        syms: vec![],
        pool: Vec::new(),
        reps: Vec::new(),
        catch_target_pos: Vec::new(),
    };

    let mut vm = vm::VM::new_by_raw_irep(irep);
    let value = vm.run()?;
    println!("{:?}", value);
    Ok(())
}
```

## Use Cases

- **Embedded Systems**: Run Ruby in resource-constrained devices
- **WebAssembly Applications**: Deploy Ruby code in browsers and serverless environments
- **Edge Computing**: Lightweight Ruby runtime for edge nodes
- **Rust Integration**: Embed Ruby scripting in Rust applications

## CLI Tool

For a command-line interface to compile and run Ruby scripts, see [mrubyedge-cli](../mrubyedge-cli).

## Documentation

- [API Documentation](https://docs.rs/mrubyedge)
- [GitHub Repository](https://github.com/mrubyedge/mrubyedge)

## License

See the [LICENSE](../LICENSE) file in the repository root.
