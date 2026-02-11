use clap::Args;
use std::io::{self, Write};

use mruby_compiler2_sys as mrbc;
use mrubyedge::yamrb::helpers::{mrb_call_inspect, mrb_funcall};

#[derive(Args)]
pub struct ReplArgs {
    /// Show verbose output
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

pub fn execute(args: ReplArgs) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("mruby/edge REPL ({})", mrubyedge::version!());
    eprintln!("Type 'exit' or press Ctrl+D to quit");
    eprintln!("Enter empty line to execute buffered code");
    eprintln!();

    // Initialize VM with empty rite
    let empty_code = "";
    let mrb_bin = unsafe {
        let mut ctx = mrbc::MRubyCompiler2Context::new();
        ctx.compile(empty_code)?
    };
    let mut rite = mrubyedge::rite::load(&mrb_bin)?;
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);

    let stdin = io::stdin();
    let mut line_number = 1;
    let mut buffer = String::new();

    loop {
        // Print prompt based on whether we're in multi-line mode
        if buffer.is_empty() {
            print!("mrb:{}> ", line_number);
        } else {
            print!("mrb:{}* ", line_number);
        }
        io::stdout().flush()?;

        // Read line
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                eprintln!();
                break;
            }
            Ok(_) => {
                let trimmed = input.trim();

                // Check for exit command
                if trimmed == "exit" || trimmed == "quit" {
                    if buffer.is_empty() {
                        break;
                    } else {
                        eprintln!("Warning: discarding buffered code");
                        buffer.clear();
                        break;
                    }
                }

                // Empty line executes the buffer
                if trimmed.is_empty() {
                    if buffer.is_empty() {
                        // Empty line with empty buffer, just continue
                        continue;
                    }

                    // Execute buffered code
                    unsafe {
                        let mut ctx = mrbc::MRubyCompiler2Context::new();
                        match ctx.compile(&buffer) {
                            Ok(mrb_bin) => match mrubyedge::rite::load(&mrb_bin) {
                                Ok(mut new_rite) => match vm.eval_rite(&mut new_rite) {
                                    Ok(result) => match mrb_call_inspect(&mut vm, result) {
                                        Ok(inspect_result) => {
                                            match TryInto::<String>::try_into(
                                                inspect_result.as_ref(),
                                            ) {
                                                Ok(s) => println!(" => {}", s),
                                                Err(_) => println!(" => <unprintable>"),
                                            }
                                        }
                                        Err(_) => println!(" => <inspect failed>"),
                                    },
                                    Err(e) => {
                                        eprintln!("{:?}", e);
                                        vm.exception.take();
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Failed to load bytecode: {:?}", e);
                                }
                            },
                            Err(e) => {
                                eprintln!("Compilation error: {}", e);
                            }
                        }
                    }

                    buffer.clear();
                    line_number += 1;
                } else {
                    // Add line to buffer
                    buffer.push_str(&input);
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    if args.verbose {
        eprintln!("REPL session ended");
    }

    Ok(())
}
