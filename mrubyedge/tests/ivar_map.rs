// Regression tests for the open-addressing IvarMap. The table stores each
// key's FNV hash and grows by load factor, so these pin that many writes on
// one object (forcing several grows) still read back exactly, that class
// variables keep working through the new map, and that the plain &str
// set_ivar entry point still round-trips.

extern crate mrubyedge;

mod helpers;

use helpers::*;
use mrubyedge::yamrb::value::RObject;
use std::rc::Rc;

fn run_i64(code: &'static str, fname: &'static str) -> i64 {
    let binary = mrbc_compile(fname, code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let args = vec![];
    mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap()
}

#[test]
fn attr_round_trip_survives_table_growth() {
    // 40 ivars on one object force the table to grow several times
    // (4 -> 8 -> 16 -> 32 -> 64 slots); every attr reader/writer pair must
    // still address its own slot afterwards.
    let mut attrs = String::new();
    let mut init = String::new();
    let mut bump = String::new();
    let mut sum = String::new();
    for i in 0..40 {
        attrs.push_str(&format!("attr_accessor :v{i}\n"));
        init.push_str(&format!("@v{i} = {};\n", i * 2));
        bump.push_str(&format!("self.v{i} = self.v{i} + 1\n"));
        if i > 0 {
            sum.push_str(" + ");
        }
        sum.push_str(&format!("self.v{i}"));
    }
    let code: &'static str = Box::leak(
        format!(
            r#"
    class Store
      {attrs}
      def initialize
        {init}
      end
      def bump_all
        {bump}
        nil
      end
      def read_all
        {sum}
      end
    end
    def test_main
      s = Store.new
      s.bump_all
      s.read_all
    end
    "#
        )
        .into(),
    );
    // sum((2i + 1) for i in 0..40)
    assert_eq!(run_i64(code, "ivar_attr_growth"), 1600);
}

#[test]
fn class_variables_read_back_after_writes() {
    let code = r#"
    class Counter
      @@count = 0
      def self.inc
        @@count += 1
      end
      def self.count
        @@count
      end
    end
    def test_main
      Counter.inc
      Counter.inc
      Counter.inc
      Counter.count
    end
    "#;
    assert_eq!(run_i64(code, "ivar_cvar_rw"), 3);
}

#[test]
fn set_ivar_accepts_str_key() {
    let obj = Rc::new(RObject::nil());
    obj.set_ivar("@name", int(7));
    let got: i64 = obj.get_ivar("@name").as_ref().try_into().unwrap();
    assert_eq!(got, 7);
    // Overwriting the same key replaces the value without growing.
    obj.set_ivar("@name", int(9));
    let got: i64 = obj.get_ivar("@name").as_ref().try_into().unwrap();
    assert_eq!(got, 9);
}
