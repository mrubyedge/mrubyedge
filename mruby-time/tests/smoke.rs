extern crate mrubyedge;
extern crate mrubyedge_time as mruby_time;

mod helpers;
use helpers::*;

/// Helper: build a VM with Time initialized and a fixed Time.__source returning [sec, nsec].
fn make_vm_with_time_source(sec: i64, nsec: u32) -> mrubyedge::yamrb::vm::VM {
    use mrubyedge::yamrb::helpers::mrb_define_singleton_cmethod;
    use mrubyedge::yamrb::value::RObject;

    // We need a dummy bytecode to open a VM. Use a trivial "nil" script.
    let dummy = mrbc_compile("dummy_source", "nil");
    let mut rite = mrubyedge::rite::load(&dummy).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    // Override Time.__source to return [sec, nsec, utc_offset]
    let sec_val = sec;
    let nsec_val = nsec;
    let time_class_obj = vm.get_const_by_name("Time").expect("Time not found");
    mrb_define_singleton_cmethod(
        &mut vm,
        time_class_obj,
        "__source",
        Box::new(move |_vm, _args| {
            let arr = vec![
                RObject::integer(sec_val).to_refcount_assigned(),
                RObject::integer(nsec_val as i64).to_refcount_assigned(),
                RObject::integer(0).to_refcount_assigned(), // utc_offset = 0
            ];
            Ok(RObject::array(arr).to_refcount_assigned())
        }),
    );

    vm
}

// 1970-01-01 00:00:00 UTC (UNIX epoch)
#[allow(unused)]
const EPOCH_SEC: i64 = 0;
// 2024-03-14 15:09:26 UTC  (Pi Day!)
// date -d "2024-03-14 15:09:26 UTC" +%s  => 1710428966
const PI_DAY_SEC: i64 = 1710428966;
// 2009-01-03 18:15:05 UTC (Bitcoin genesis block)
#[allow(unused)]
const GENESIS_SEC: i64 = 1231006505;

#[test]
fn test_time_at_epoch_year() {
    let code = "Time.at(0).year";
    let binary = mrbc_compile("time_epoch_year", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let year: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(year, 1970);
}

#[test]
fn test_time_at_epoch_month() {
    let code = "Time.at(0).month";
    let binary = mrbc_compile("time_epoch_month", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 1);
}

#[test]
fn test_time_at_epoch_day() {
    let code = "Time.at(0).day";
    let binary = mrbc_compile("time_epoch_day", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 1);
}

#[test]
fn test_time_at_epoch_wday() {
    // 1970-01-01 was Thursday (wday=4)
    let code = "Time.at(0).wday";
    let binary = mrbc_compile("time_epoch_wday", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 4); // Thursday
}

#[test]
fn test_time_at_epoch_hour_min_sec() {
    let code = "[Time.at(0).hour, Time.at(0).min, Time.at(0).sec]";
    let binary = mrbc_compile("time_epoch_hms", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let arr = match &result.value {
        mrubyedge::yamrb::value::RValue::Array(a) => a.borrow().clone(),
        _ => panic!("expected array"),
    };
    assert_eq!(arr.len(), 3);
    let h: i64 = arr[0].as_ref().try_into().unwrap();
    let m: i64 = arr[1].as_ref().try_into().unwrap();
    let s: i64 = arr[2].as_ref().try_into().unwrap();
    assert_eq!(h, 0);
    assert_eq!(m, 0);
    assert_eq!(s, 0);
}

#[test]
fn test_time_pi_day() {
    // 2024-03-14 15:09:26 UTC
    let code = format!(
        "[Time.at({}).year, Time.at({}).month, Time.at({}).day, Time.at({}).hour, Time.at({}).min, Time.at({}).sec]",
        PI_DAY_SEC, PI_DAY_SEC, PI_DAY_SEC, PI_DAY_SEC, PI_DAY_SEC, PI_DAY_SEC
    );
    let binary = mrbc_compile("time_pi_day", &code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let arr = match &result.value {
        mrubyedge::yamrb::value::RValue::Array(a) => a.borrow().clone(),
        _ => panic!("expected array"),
    };
    assert_eq!(arr.len(), 6);
    let year: i64 = arr[0].as_ref().try_into().unwrap();
    let month: i64 = arr[1].as_ref().try_into().unwrap();
    let day: i64 = arr[2].as_ref().try_into().unwrap();
    let hour: i64 = arr[3].as_ref().try_into().unwrap();
    let min: i64 = arr[4].as_ref().try_into().unwrap();
    let sec: i64 = arr[5].as_ref().try_into().unwrap();
    assert_eq!(year, 2024);
    assert_eq!(month, 3);
    assert_eq!(day, 14);
    assert_eq!(hour, 15);
    assert_eq!(min, 9);
    assert_eq!(sec, 26);
}

#[test]
fn test_time_nsec() {
    let code = "Time.at(0, 123456789).nsec";
    let binary = mrbc_compile("time_nsec", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 123456789);
}

#[test]
fn test_time_to_s() {
    // 1970-01-01 00:00:00 UTC
    let code = "Time.at(0).to_s";
    let binary = mrbc_compile("time_to_s", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let s: String = result.as_ref().try_into().unwrap();
    assert_eq!(s, "1970-01-01 00:00:00 +0000");
}

#[test]
fn test_time_to_s_with_offset() {
    // 1970-01-01 00:00:00 UTC, localtime with +32400 (JST +09:00)
    let code = "Time.at(0).localtime(32400).to_s";
    let binary = mrbc_compile("time_to_s_offset", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let s: String = result.as_ref().try_into().unwrap();
    assert_eq!(s, "1970-01-01 09:00:00 +0900");
}

#[test]
fn test_time_add() {
    // epoch + 3661 = 1970-01-01 01:01:01 UTC
    let code = "(Time.at(0) + 3661).to_s";
    let binary = mrbc_compile("time_add", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let s: String = result.as_ref().try_into().unwrap();
    assert_eq!(s, "1970-01-01 01:01:01 +0000");
}

#[test]
fn test_time_sub_seconds() {
    // epoch + 3600 - 1800 = 1970-01-01 00:30:00 UTC
    let code = "(Time.at(3600) - 1800).to_s";
    let binary = mrbc_compile("time_sub", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let s: String = result.as_ref().try_into().unwrap();
    assert_eq!(s, "1970-01-01 00:30:00 +0000");
}

#[test]
fn test_time_sub_times() {
    // Time.at(3600) - Time.at(0) = 3600.0
    let code = "Time.at(3600) - Time.at(0)";
    let binary = mrbc_compile("time_sub_times", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let f: f64 = result.as_ref().try_into().unwrap();
    assert!((f - 3600.0).abs() < 1e-6);
}

#[test]
fn test_time_cmp() {
    let code = "Time.at(100) <=> Time.at(50)";
    let binary = mrbc_compile("time_cmp_gt", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 1);
}

#[test]
fn test_time_cmp_eq() {
    let code = "Time.at(100) <=> Time.at(100)";
    let binary = mrbc_compile("time_cmp_eq", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 0);
}

#[test]
fn test_time_cmp_lt() {
    let code = "Time.at(50) <=> Time.at(100)";
    let binary = mrbc_compile("time_cmp_lt", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, -1);
}

#[test]
fn test_time_now() {
    // Time.now uses Time.__source; default impl is tested on non-wasm
    // Just verify we get a Time object back (year >= 2020)
    let code = "Time.now.year";
    let binary = mrbc_compile("time_now_year", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let year: i64 = result.as_ref().try_into().unwrap();
    assert!(year >= 2020, "year should be >= 2020, got {}", year);
}

#[test]
fn test_time_now_overridden_source() {
    // Override Time.__source to return a fixed time (Pi Day 2024)
    let code = "Time.now.year";
    let binary = mrbc_compile("time_now_override", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = make_vm_with_fixed_source(&mut rite, PI_DAY_SEC, 0);

    let result = vm.run().unwrap();
    let year: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(year, 2024);
}

fn make_vm_with_fixed_source(
    rite: &mut mrubyedge::rite::Rite,
    sec: i64,
    nsec: u32,
) -> mrubyedge::yamrb::vm::VM {
    use mrubyedge::yamrb::helpers::mrb_define_singleton_cmethod;
    use mrubyedge::yamrb::value::RObject;

    let mut vm = mrubyedge::yamrb::vm::VM::open(rite);
    mruby_time::init_time(&mut vm);

    let time_class_obj = vm.get_const_by_name("Time").expect("Time not found");
    mrb_define_singleton_cmethod(
        &mut vm,
        time_class_obj,
        "__source",
        Box::new(move |_vm, _args| {
            let arr = vec![
                RObject::integer(sec).to_refcount_assigned(),
                RObject::integer(nsec as i64).to_refcount_assigned(),
                RObject::integer(0).to_refcount_assigned(), // utc_offset = 0
            ];
            Ok(RObject::array(arr).to_refcount_assigned())
        }),
    );

    vm
}

#[test]
fn test_time_utc_offset() {
    let code = "Time.at(0).utc_offset";
    let binary = mrbc_compile("time_utc_offset", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 0);
}

#[test]
fn test_time_localtime_offset() {
    // JST = UTC+9 = +32400 seconds
    let code = "Time.at(0).localtime(32400).utc_offset";
    let binary = mrbc_compile("time_localtime", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 32400);
}

#[test]
fn test_time_to_i() {
    let code = "Time.at(12345).to_i";
    let binary = mrbc_compile("time_to_i", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 12345);
}

#[test]
fn test_time_to_f() {
    let code = "Time.at(12345, 500000000).to_f";
    let binary = mrbc_compile("time_to_f", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: f64 = result.as_ref().try_into().unwrap();
    assert!((v - 12345.5).abs() < 1e-6);
}

#[test]
fn test_pi_day_wday() {
    // 2024-03-14 was a Thursday (wday=4)
    let code = format!("Time.at({}).wday", PI_DAY_SEC);
    let binary = mrbc_compile("time_pi_day_wday", &code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_time::init_time(&mut vm);

    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 4); // Thursday
}
