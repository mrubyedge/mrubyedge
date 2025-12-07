extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "lib.rs.tmpl", escape = "none")]
pub struct LibRs<'a> {
    pub file_basename: &'a str,

    pub ftypes: &'a [RustFnTemplate<'a>],
    pub ftypes_imports: &'a [RustImportFnTemplate<'a>],
}

pub struct RustFnTemplate<'a> {
    pub func_name: &'a str,
    pub args_decl: &'a str,
    pub args_let_vec: &'a str,
    pub str_args_converter: &'a str,
    pub rettype_decl: &'a str,
    pub handle_retval: &'a str,
    pub exported_helper_var: &'a str,
}

pub struct RustImportFnTemplate<'a> {
    pub func_name: &'a str,
    pub args_decl: &'a str,
    pub imported_body: &'a str,
    pub rettype_decl: &'a str,
    pub import_helper_var: &'a str,
}

#[test]
fn test_lib_rs_template() {
    use crate::rbs_parser::parse;

    let def = "
def foo_bar: (Integer) -> Integer
";

    let ret = parse(def).unwrap();
    let ftype = ret.1;
    let ftypes = vec![RustFnTemplate {
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
    if std::env::var("VERBOSE").is_ok() {
        println!("{}", &rendered);
    }

    assert!(syn::parse_file(&rendered).is_ok());
}
