use clap::Args;
use std::{fs::File, io::Read, path::PathBuf};

use mruby_compiler2_sys as mrbc;
use mrubyedge;

#[derive(Args)]
pub struct RunArgs {
    /// Dump instruction sequences
    #[arg(long)]
    pub dump_insns: bool,

    /// Execute the given Ruby code string
    #[arg(short = 'e', long = "eval", value_name = "CODE")]
    pub eval: Option<String>,

    /// Ruby source file or mrb binary to run
    pub file: Option<PathBuf>,
}

pub fn execute(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    let buf = if let Some(code) = &args.eval {
        // Execute code from -e option
        code.clone().into_bytes()
    } else if let Some(file) = args.file {
        // Read from file
        let mut buf = Vec::new();
        File::open(&file)?.read_to_end(&mut buf)?;
        buf
    } else {
        return Err("Either -e option or file path must be provided".into());
    };

    let is_mrb_direct =
        args.eval.is_none() && buf.len() >= 4 && buf[0..4] == [b'R', b'I', b'T', b'E'][..];
    unsafe {
        let mrb_bin = if is_mrb_direct {
            buf.to_vec()
        } else {
            let buf = String::from_utf8(buf)?;
            let mut ctx = mrbc::MRubyCompiler2Context::new();
            if args.dump_insns {
                ctx.dump_bytecode(&buf)?;
            }
            ctx.compile(&buf)?
        };
        let mut rite = mrubyedge::rite::load(&mrb_bin)?;
        let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
        vm.run()?;
    }

    Ok(())
}
