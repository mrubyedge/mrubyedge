extern crate mruby_compiler2_sys;
extern crate rand;

use clap::Args;
use std::{
    env,
    fs::{File, rename},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use askama::Template;
use rand::distributions::{Alphanumeric, DistString};

use crate::rbs_parser;
use crate::template;

const MRUBY_EDGE_DEFAULT_VERSION: &str = ">= 1";

#[derive(Debug, Clone, Args)]
pub struct WasmArgs {
    #[arg(short = 'f', long)]
    fnname: Option<String>,
    #[arg(short = 'm', long)]
    mruby_edge_version: Option<String>,
    #[arg(short = 'F', long)]
    features: Vec<String>,
    #[arg(short = 'W', long)]
    no_wasi: bool,
    #[arg(short = 'o', long)]
    out_path: Option<PathBuf>,
    #[arg(long)]
    skip_cleanup: bool,
    #[arg(long)]
    debug_mruby_edge: bool,
    #[arg(long)]
    verbose: bool,
    #[arg(long)]
    strip_binary: bool,
    path: PathBuf,
}

fn sh_do(sharg: &str, debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("running: `{}`", sharg);
    let out = Command::new("/bin/sh").args(["-c", sharg]).output()?;
    if debug && !out.stdout.is_empty() {
        println!(
            "stdout:\n{}",
            String::from_utf8_lossy(&out.stdout).to_string().trim()
        );
    }
    if debug && !out.stderr.is_empty() {
        println!(
            "stderr:\n{}",
            String::from_utf8_lossy(&out.stderr).to_string().trim()
        );
    }
    if !out.status.success() {
        println!("{:?}", out.status);
        panic!("failed to execute command");
    }

    Ok(())
}

fn file_prefix_of(file: &Path) -> Option<String> {
    file.file_name()?
        .to_str()?
        .split('.')
        .next()
        .map(|s| s.to_string())
}

fn debug_println(debug: bool, msg: &str) {
    if debug {
        eprintln!("{}", msg);
    }
}

pub fn execute(args: WasmArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let suffix = Alphanumeric.sample_string(&mut rng, 32);

    let fnname = args.fnname;
    let path = args.path;
    let mrubyfile = std::fs::canonicalize(&path)?;
    let fname = file_prefix_of(mrubyfile.as_path()).unwrap();

    let pwd = std::env::current_dir()?;
    std::env::set_current_dir(std::env::var("TMPDIR").unwrap_or("/tmp".to_string()))?;

    let dirname = format!("work-mrubyedge-{}", suffix);
    std::fs::create_dir(&dirname)?;
    std::env::set_current_dir(format!("./work-mrubyedge-{}", &suffix))?;
    std::fs::create_dir("src")?;

    let code = std::fs::read_to_string(&mrubyfile)?;
    let out_file = format!("src/{}.mrb", fname);

    if args.verbose {
        unsafe {
            let mut context = mruby_compiler2_sys::MRubyCompiler2Context::new();
            context.dump_bytecode(&code)?;
        }
    }
    unsafe {
        mruby_compiler2_sys::MRubyCompiler2Context::new()
            .compile_to_file(&code, out_file.as_ref())?
    }

    let mut features = Vec::new();
    if args.no_wasi {
        features.push("no-wasi");
    } else {
        features.push("wasi");
    }
    for f in args.features.iter() {
        features.push(f.as_str());
    }
    let mrubyedge_feature = features
        .iter()
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<String>>()
        .join(", ");

    if args.debug_mruby_edge {
        let mruby_edge_crate_path = env::var("MRUBYEDGE_LOCAL_CRATE_PATH").unwrap_or_else(|_| {
            "/Users/udzura/ghq/github.com/mrubyedge/mrubyedge/mrubyedge".to_string()
        });
        let cargo_toml = template::cargo_toml::CargoTomlDebug {
            mruby_edge_crate_path: &mruby_edge_crate_path,
            mrubyedge_feature: &mrubyedge_feature,
        };
        std::fs::write("Cargo.toml", cargo_toml.render()?)?;
    } else {
        let cargo_toml = template::cargo_toml::CargoToml {
            mrubyedge_version: &args
                .mruby_edge_version
                .unwrap_or_else(|| MRUBY_EDGE_DEFAULT_VERSION.to_string()),
            mrubyedge_feature: &mrubyedge_feature,
            strip: &args.strip_binary.to_string(),
        };
        std::fs::write("Cargo.toml", cargo_toml.render()?)?;
    }

    let export_rbs_fname = format!("{}.export.rbs", fname);
    let export_rbs = mrubyfile.parent().unwrap().join(&export_rbs_fname);

    let mut ftypes_imports = Vec::new();
    let import_rbs_fname = format!("{}.import.rbs", fname);
    let import_rbs = mrubyfile.parent().unwrap().join(&import_rbs_fname);
    if import_rbs.exists() {
        debug_println(
            args.verbose,
            &format!(
                "detected import.rbs: {}",
                import_rbs.as_path().to_string_lossy()
            ),
        );
        let mut f = File::open(import_rbs)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;

        let (_, parsed) = rbs_parser::parse(&s).unwrap();
        let parsed: &mut [rbs_parser::FuncDef] = Vec::leak(parsed);
        for def in parsed.iter() {
            ftypes_imports.push(template::RustImportFnTemplate {
                func_name: &def.name,
                args_decl: def.args_decl(),
                rettype_decl: def.rettype_decl(),
                imported_body: def.imported_body(),
                import_helper_var: def.import_helper_var(),
            })
        }
    }

    let cont = if export_rbs.exists() {
        debug_println(
            args.verbose,
            &format!(
                "detected export.rbs: {}",
                export_rbs.as_path().to_string_lossy()
            ),
        );
        let mut f = File::open(export_rbs)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;

        let (_, parsed) = rbs_parser::parse(&s).unwrap();
        let mut ftypes = vec![];
        let parsed: &mut [rbs_parser::FuncDef] = Vec::leak(parsed);
        for def in parsed.iter() {
            ftypes.push(template::RustFnTemplate {
                func_name: &def.name,
                args_decl: def.args_decl(),
                args_let_vec: def.args_let_vec(),
                str_args_converter: def.str_args_converter(),
                rettype_decl: def.rettype_decl(),
                handle_retval: def.handle_retval(),
                exported_helper_var: def.exported_helper_var(),
            })
        }

        let lib_rs = template::LibRs {
            file_basename: &fname,
            ftypes: &ftypes,
            ftypes_imports: &ftypes_imports,
        };

        lib_rs.render()?
    } else {
        if fnname.is_none() {
            panic!("--fnname FNNAME should be specified when export.rbs does not exist")
        }
        let fnname = fnname.unwrap();

        let ftypes = vec![template::RustFnTemplate {
            func_name: &fnname,
            args_decl: "",
            args_let_vec: "vec![]",
            str_args_converter: "",
            rettype_decl: "-> ()",
            handle_retval: "()",
            exported_helper_var: "",
        }];

        let lib_rs = template::LibRs {
            file_basename: &fname,
            ftypes: &ftypes,
            ftypes_imports: &ftypes_imports,
        };

        lib_rs.render()?
    };
    debug_println(args.verbose, "[debug] will generate main.rs:");
    debug_println(args.verbose, &cont);
    std::fs::write("src/lib.rs", cont)?;

    let target = if args.no_wasi {
        "wasm32-unknown-unknown"
    } else {
        "wasm32-wasip1"
    };

    sh_do(
        &format!("cargo build --target {} --release", target),
        args.verbose,
    )?;

    let output_path = if let Some(out_path) = &args.out_path {
        std::fs::canonicalize(out_path).unwrap_or_else(|_| pwd.join(out_path))
    } else {
        pwd.join(format!("{}.wasm", &fname))
    };

    let from = format!("./target/{}/release/mywasm.wasm", target);
    let to = output_path.to_str().expect("Invalid output path");
    rename(from, to)?;
    if args.skip_cleanup {
        println!(
            "debug: working directory for compile wasm is remained in {}",
            std::env::current_dir()?.as_os_str().to_str().unwrap()
        );
    } else {
        std::env::set_current_dir("..")?;
        sh_do(&format!("rm -rf {}", &dirname), args.verbose)?;
    }

    std::env::set_current_dir(pwd)?;

    println!(
        "[ok] wasm file is generated: {}",
        &output_path.to_string_lossy()
    );

    Ok(())
}
