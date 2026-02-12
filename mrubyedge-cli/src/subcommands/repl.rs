use clap::Args;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::{
    io::{self, Write},
    rc::Rc,
};

use mruby_compiler2_sys as mrbc;
use mrubyedge::{
    RObject,
    yamrb::{helpers::mrb_call_inspect, value::RHashMap},
};

#[derive(Args)]
pub struct ReplArgs {
    /// Show verbose output
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

pub fn execute(args: ReplArgs) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("mruby/edge REPL ({})", mrubyedge::version!());
    eprintln!("Type 'exit[↩]' or press Ctrl+D to quit");
    eprintln!(
        "Press Enter to execute, Option+Enter(Shift+Enter also supported in iTerm2) for line continuation"
    );
    eprintln!();

    // Initialize VM with empty rite
    let empty_code = "";
    let mrb_bin = unsafe {
        let mut ctx = mrbc::MRubyCompiler2Context::new();
        ctx.compile(empty_code)?
    };
    let mut rite = mrubyedge::rite::load(&mrb_bin)?;
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);

    // Enable raw mode
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    let mut line_number = 1;
    let mut buffer = String::new();
    let mut current_line = String::new();

    // Print initial prompt
    print!("repl:{:03}> ", line_number);
    stdout.flush()?;
    let mut top_level_lvars: RHashMap<String, Rc<RObject>> = RHashMap::default();

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Read key event
            if let Event::Key(key_event) = event::read()? {
                match key_event {
                    // Ctrl+D - Exit
                    KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        terminal::disable_raw_mode()?;
                        println!();
                        break;
                    }
                    // Ctrl+C - Clear current line
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        terminal::disable_raw_mode()?;
                        println!();
                        current_line.clear();
                        buffer.clear();
                        line_number += 1;
                        print!("repl:{:03}> ", line_number);
                        stdout.flush()?;
                        terminal::enable_raw_mode()?;
                    }
                    // Alt+Enter (Option+Enter) - Add line to buffer and continue
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::ALT | KeyModifiers::SHIFT,
                        ..
                    } => {
                        terminal::disable_raw_mode()?;
                        println!();
                        buffer.push_str(&current_line);
                        buffer.push('\n');
                        current_line.clear();
                        print!("repl:{:03}* ", line_number);
                        stdout.flush()?;
                        terminal::enable_raw_mode()?;
                    }
                    // Regular Enter - Execute buffer
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        terminal::disable_raw_mode()?;
                        println!();

                        // Add current line to buffer
                        if !current_line.is_empty() {
                            buffer.push_str(&current_line);
                            buffer.push('\n');
                        }

                        if buffer.trim().is_empty() {
                            current_line.clear();
                            print!("repl:{:03}> ", line_number);
                            stdout.flush()?;
                            terminal::enable_raw_mode()?;
                            continue;
                        }

                        // Check for exit command
                        let trimmed = buffer.trim();
                        if trimmed == "exit" || trimmed == "quit" {
                            break;
                        }

                        // Execute buffered code
                        unsafe {
                            let mut ctx = mrbc::MRubyCompiler2Context::new();
                            if args.verbose {
                                ctx.dump_bytecode(&buffer).unwrap();
                            }
                            match ctx.compile(&buffer) {
                                Ok(mrb_bin) => match mrubyedge::rite::load(&mrb_bin) {
                                    Ok(mut new_rite) => {
                                        // FIXME: sub ireps's lv not handled yet
                                        let top_rep = &new_rite.irep[0];
                                        for (reg, name) in top_rep.lv.iter().enumerate() {
                                            if let Some(name) = name
                                                && let Some(value) = top_level_lvars
                                                    .get(&name.to_string_lossy().to_string())
                                            {
                                                vm.regs[reg + 1] = value.clone().into();
                                            }
                                        }
                                        match vm.eval_rite(&mut new_rite) {
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
                                        }
                                        if let Some(lv) = &vm.current_irep.lv {
                                            for (reg, name) in lv.iter() {
                                                let value =
                                                    vm.regs[*reg].as_ref().cloned().unwrap_or(
                                                        RObject::nil().to_refcount_assigned(),
                                                    );
                                                top_level_lvars
                                                    .insert(name.to_string(), value.clone());
                                            }
                                            for (k, v) in top_level_lvars.iter() {
                                                let inspect: String =
                                                    mrb_call_inspect(&mut vm, v.clone())
                                                        .unwrap()
                                                        .as_ref()
                                                        .try_into()
                                                        .unwrap();
                                                if args.verbose {
                                                    eprintln!("  [lv] {} => {}", k, inspect);
                                                }
                                            }
                                        }
                                    }
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
                        current_line.clear();
                        line_number += 1;
                        print!("repl:{:03}> ", line_number);
                        stdout.flush()?;

                        terminal::enable_raw_mode()?;
                    }
                    // Backspace
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        if !current_line.is_empty() {
                            current_line.pop();
                            execute!(
                                stdout,
                                cursor::MoveLeft(1),
                                terminal::Clear(ClearType::UntilNewLine)
                            )?;
                            stdout.flush()?;
                        }
                    }
                    // Regular character input
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                        ..
                    } => {
                        current_line.push(c);
                        print!("{}", c);
                        stdout.flush()?;
                    }
                    _ => {
                        // Ignore unhandled key events
                    }
                }
            }
        }

        Ok(())
    })();

    // Disable raw mode
    terminal::disable_raw_mode()?;

    if args.verbose {
        eprintln!("REPL session ended");
    }

    result
}
