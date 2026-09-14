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
    assert_eq!(&rite.binary_header.major_version, b"04");
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

#[test]
fn test_rite_parse_bigint_pool_value() {
    let code = r#"
    x = 99999999999999999999999
    x
    "#;
    let binary = mrbc_compile("bigint", code);

    let rite = mrubyedge::rite::load(&binary).unwrap();

    use mrubyedge::rite::PoolValue;
    let bigint = rite.irep[0]
        .pool
        .iter()
        .find_map(|p| match p {
            PoolValue::BigInt(bytes) => Some(bytes),
            _ => None,
        })
        .expect("Should have a BigInt in pool");

    // `[base][digits]`: base 10 followed by the 23 decimal digits.
    assert_eq!(bigint.len(), 24);
    assert_eq!(bigint[0], 10);
    assert!(bigint[1..].iter().all(|&b| b == b'9'));
}

#[test]
fn test_rite_truncated_bigint_is_error_not_panic() {
    let binary = mrbc_compile("bigint", "x = 99999999999999999999999");
    let digits = [b'9'; 23];
    let digits_start = binary
        .windows(digits.len())
        .position(|w| w == digits)
        .expect("Should find the bigint digits");

    // Layout before the digits is [type=7][len:u8][base:i8].
    let len_pos = digits_start - 2;
    let mut chunk = binary.clone();
    chunk[len_pos] = 0xff;

    // An out-of-bounds length must be reported, not panic.
    assert!(mrubyedge::rite::load(&chunk).is_err());
}

#[test]
fn a_chunk_of_an_unknown_format_version_is_refused_test() {
    let binary = mrbc_compile("compiled", "1 + 1");

    // Bytes 4 and 5 of the header are the major version. mruby 3.x chunks are
    // refused too: the opcode numbering is not the one this VM decodes.
    for major in [b"02", b"03", b"05"] {
        let mut chunk = binary.clone();
        chunk[4..6].copy_from_slice(major);

        // Assert
        let err = mrubyedge::rite::load(&chunk).unwrap_err();
        assert!(matches!(err, mrubyedge::rite::Error::UnsupportedVersion(m) if &m == major));
    }
}
