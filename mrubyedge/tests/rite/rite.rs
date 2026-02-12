extern crate mec_mrbc_sys;
extern crate mrubyedge;

use super::helpers::*;

#[test]
fn test_rite_parse_hello_world() {
    let code = r#"
    puts "Hello, World!"
    "#;
    let binary = mrbc_compile("hello", code);
    let rite = mrubyedge::rite::load(&binary).unwrap();

    // Check binary header
    assert_eq!(&rite.binary_header.ident, b"RITE");
    assert_eq!(&rite.binary_header.major_version, b"03");
    assert_eq!(&rite.binary_header.minor_version, b"00");

    // Check IREP section
    assert_eq!(rite.irep.len(), 1);
    let irep = &rite.irep[0];

    // Should have at least some instructions
    assert!(!irep.insn.is_empty());

    // Should have some pool values (for the string)
    assert!(irep.plen > 0);
}

#[test]
fn test_rite_parse_with_local_variables() {
    let code = r#"
    def greet(name)
      message = "Hello, #{name}!"
      puts message
      message
    end
    
    greet("Alice")
    "#;
    let binary = mrbc_compile("greet", code);
    let rite = mrubyedge::rite::load(&binary).unwrap();

    // Check binary header
    assert_eq!(&rite.binary_header.ident, b"RITE");

    // Should have multiple ireps (main + function definition)
    assert!(rite.irep.len() == 2);
    assert!(rite.irep[0].lv.is_empty());
    assert!(rite.irep[1].lv.len() == 3); // name, (&nil), message

    // Check LVAR section exists
    let lvar = rite.lvar.unwrap();
    assert!(lvar.syms.len() == 2);

    let name = rite.irep[1].lv[0].as_ref().cloned().unwrap();
    assert_eq!(&name.to_string_lossy(), "name");

    let message = rite.irep[1].lv[2].as_ref().cloned().unwrap();
    assert_eq!(&message.to_string_lossy(), "message");
}

#[test]
fn test_rite_parse_pool_values() {
    let code = r#"
    a = "string"
    b = 42
    c = 3.14
    d = 9999999999
    [a, b, c, d]
    "#;
    let binary = mrbc_compile("pool", code);
    let rite = mrubyedge::rite::load(&binary).unwrap();

    assert_eq!(&rite.binary_header.ident, b"RITE");

    let irep = &rite.irep[0];
    assert!(rite.lvar.unwrap().syms.len() == 4);

    // Should have multiple pool values
    assert!(irep.pool.len() == 3);
    assert!(irep.lv.len() == 4);

    // Check pool value types
    use mrubyedge::rite::PoolValue;
    let has_string = irep
        .pool
        .iter()
        .any(|p| matches!(p, PoolValue::Str(_) | PoolValue::SStr(_)));
    let has_float = irep.pool.iter().any(|p| matches!(p, PoolValue::Float(_)));
    let has_int64 = irep.pool.iter().any(|p| matches!(p, PoolValue::Int64(_)));

    assert!(has_string, "Should have string in pool");
    assert!(has_float, "Should have float in pool");
    assert!(has_int64, "Should have int64 in pool");
}
