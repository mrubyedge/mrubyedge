use askama::Template;
use mec::rbs_parser::*;
use mec::template::LibRs;

fn main() {
    let def = "
def foo_bar: (Integer) -> Integer
";

    let ret = parse(def).unwrap();
    let ftype = ret.1;
    let ftypes = vec![mec::template::RustFnTemplate {
        func_name: &ftype[0].name,
        args_decl: "a: i32",
        args_let_vec: "vec![std::rc::Rc::new(RObject::RInteger(a as i64))]",
        rettype_decl: "-> i32",
        str_args_converter: "// do nothing",
        handle_retval: "5471",
        exported_helper_var: "",
    }];
    let imports = vec![];

    let lib_rs = LibRs {
        file_basename: "world",
        ftypes: &ftypes,
        ftypes_imports: &imports,
    };

    let rendered = lib_rs.render().unwrap();
    println!("{}", &rendered);

    // Check if rendered is valid Rust syntax using syn
    println!("\n--- Checking if rendered code is valid Rust syntax ---");
    
    match syn::parse_file(&rendered) {
        Ok(_) => {
            println!("✓ Rendered code has valid Rust syntax!");
        }
        Err(e) => {
            println!("✗ Rendered code has syntax errors:");
            println!("{}", e);
        }
    }
}
