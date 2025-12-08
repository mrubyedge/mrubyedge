extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn module_definition_test() {
    let script = r#"
module TestModule
  def module_method
    42
  end
end

TestModule
"#;

    let binary = mrbc_compile("module_def", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    
    // Result should be the module itself
    assert!(matches!(result.tt, mrubyedge::yamrb::value::RType::Module));
}

