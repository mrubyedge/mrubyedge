use std::env;
use std::fs::remove_file;
use std::process::Command;
use std::rc::Rc;

use mrubyedge::RObject;
use mrubyedge::yamrb::helpers::mrb_call_p;

extern crate mrubyedge;

fn compile(
    source_code: &str,
    output_path: &str,
    is_verbose: bool,
) -> Result<Vec<u8>, std::io::Error> {
    let source_path = output_path.replace(".mrb", ".rb");
    std::fs::write(&source_path, source_code)?;
    let mut mrbc = Command::new("mrbc");
    if is_verbose {
        mrbc.arg("-v");
    }
    let result = mrbc
        .arg("-o")
        .arg(output_path)
        .arg(&source_path)
        .output()
        .expect("failed to compile mruby script");
    if is_verbose {
        eprintln!("stdout: {}", String::from_utf8_lossy(&result.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&result.stderr));
    }
    std::fs::read(output_path)
}

fn result_p(vm: &mut mrubyedge::VM, result: Rc<RObject>) {
    eprint!("return value: ");
    mrb_call_p(vm, result);
}

fn main() -> Result<(), std::io::Error> {
    let is_verbose = env::var("MRUBYEDGE_DEBUG").is_ok();

    let code1 = r#"
    class Foo
      def bar
        puts "Hello from Foo#bar"
        42
      end
    end
    puts "Hi"
    "#;
    let output_path_1 = "/tmp/__tmp__.mrb";
    let mrb1 = compile(code1, output_path_1, is_verbose)?;
    let mut rite = mrubyedge::rite::load(&mrb1).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let _ = vm.run().unwrap();
    remove_file(output_path_1)?;

    let code = r#"
    class Bar
      def baz
        puts "Hello from Bar#baz"
        100
      end
    end
    "#;
    let output_path = "/tmp/__tmp_a__.mrb";
    let mrb = compile(code, output_path, is_verbose)?;
    let mut rite = mrubyedge::rite::load(&mrb).unwrap();
    let res = vm.eval_rite(&mut rite).unwrap();
    result_p(&mut vm, res);
    remove_file(output_path)?;

    let code = r#"
    puts "Hola"
    foo = Foo.new
    v1 = foo.bar
    bar = Bar.new
    v2 = bar.baz
    v1 + v2
    "#;
    let output_path = "/tmp/__tmp_z__.mrb";
    let mrb = compile(code, output_path, is_verbose)?;
    let mut rite = mrubyedge::rite::load(&mrb).unwrap();
    let res = vm.eval_rite(&mut rite).unwrap();
    result_p(&mut vm, res);
    remove_file(output_path)?;

    // dbg!(&vm);
    Ok(())
}
