use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod, mrb_funcall},
        value::{RData, RHashMap, RObject, RType, RValue},
        vm::VM,
    },
};

/// Type alias for datetime parts: (year, month, day, wday, hour, min, sec)
type DateTimeParts = (i32, u32, u32, u32, u32, u32, u32);

/// Rust-side representation of a Ruby Time object.
/// Stores seconds and nanoseconds since UNIX epoch, and UTC offset in seconds.
#[derive(Debug, Clone)]
pub struct RTimeData {
    /// Seconds since UNIX epoch (can be negative for times before 1970)
    pub sec: i64,
    /// Nanoseconds within the current second (0..999_999_999)
    pub nsec: u32,
    /// UTC offset in seconds (e.g. +9h = 32400, -5h = -18000)
    pub utc_offset: i32,
    /// Cached result of to_datetime_parts() — computed lazily, interior-mutable.
    cached_parts: Cell<Option<DateTimeParts>>,
}

impl RTimeData {
    pub fn new(sec: i64, nsec: u32, utc_offset: i32) -> Self {
        RTimeData {
            sec,
            nsec,
            utc_offset,
            cached_parts: Cell::new(None),
        }
    }

    /// Calculate the "local" seconds (sec + utc_offset) for date/time decomposition.
    fn local_sec(&self) -> i64 {
        self.sec + self.utc_offset as i64
    }

    /// Decompose into (year, month, day, wday, hour, min, sec_in_day).
    /// Uses the proleptic Gregorian calendar algorithm.
    /// Result is cached on first call via interior mutability.
    pub fn to_datetime_parts(&self) -> DateTimeParts {
        if let Some(parts) = self.cached_parts.get() {
            return parts;
        }
        let local = self.local_sec();

        // Time of day
        let sec_in_day = local.rem_euclid(86400) as u32;
        let hour = sec_in_day / 3600;
        let min = (sec_in_day % 3600) / 60;
        let sec = sec_in_day % 60;

        // Day number from epoch (days since 1970-01-01, can be negative)
        let days_from_epoch = local.div_euclid(86400);

        // Convert to Julian Day Number; 1970-01-01 = JDN 2440588
        let jdn = days_from_epoch + 2440588;

        // Gregorian calendar conversion from JDN
        // Algorithm from: https://en.wikipedia.org/wiki/Julian_day#Julian_day_number_calculation
        let l = jdn + 68569;
        let n = (4 * l) / 146097;
        let l = l - (146097 * n + 3) / 4;
        let i = (4000 * (l + 1)) / 1461001;
        let l = l - (1461 * i) / 4 + 31;
        let j = (80 * l) / 2447;
        let day = l - (2447 * j) / 80;
        let l = j / 11;
        let month = j + 2 - 12 * l;
        let year = 100 * (n - 49) + i + l;

        // Weekday: JDN mod 7; JDN=0 is Monday in proleptic... actually
        // 2440588 % 7 = 4, and 1970-01-01 was Thursday (wday=4 in Ruby)
        let wday = (jdn + 1).rem_euclid(7) as u32; // 0=Sunday, 1=Monday, ...

        let parts = (year as i32, month as u32, day as u32, wday, hour, min, sec);
        self.cached_parts.set(Some(parts));
        parts
    }

    /// Format as "%Y-%m-%d %H:%M:%S %z"
    pub fn to_s(&self) -> String {
        let (year, month, day, _wday, hour, min, sec) = self.to_datetime_parts();
        let offset_sign = if self.utc_offset >= 0 { '+' } else { '-' };
        let abs_offset = self.utc_offset.unsigned_abs();
        let offset_h = abs_offset / 3600;
        let offset_m = (abs_offset % 3600) / 60;
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}{:02}{:02}",
            year, month, day, hour, min, sec, offset_sign, offset_h, offset_m
        )
    }
}

/// Extract RTimeData from an RObject (must be a Data object holding RTimeData).
fn get_time_data(obj: &Rc<RObject>) -> Result<RTimeData, Error> {
    match &obj.value {
        RValue::Data(data) => {
            let borrow = data.data.borrow();
            let any_ref = borrow
                .as_ref()
                .ok_or_else(|| Error::RuntimeError("Invalid Time data".to_string()))?;
            let time = any_ref
                .downcast_ref::<RTimeData>()
                .ok_or_else(|| Error::RuntimeError("Invalid Time data".to_string()))?;
            Ok(time.clone())
        }
        _ => Err(Error::RuntimeError("Expected a Time object".to_string())),
    }
}

/// Create an Rc<RObject> wrapping an RTimeData.
fn make_time_object(vm: &mut VM, time_data: RTimeData) -> Rc<RObject> {
    let time_class_obj = vm
        .get_const_by_name("Time")
        .expect("Time class not found; did you call init_time?");
    let class = match &time_class_obj.value {
        RValue::Class(c) => c.clone(),
        _ => panic!("Time is not a class"),
    };
    let rdata = Rc::new(RData {
        class,
        data: RefCell::new(Some(Rc::new(Box::new(time_data) as Box<dyn Any>))),
        ref_count: 1,
    });
    Rc::new(RObject {
        tt: RType::Data,
        value: RValue::Data(rdata),
        object_id: Cell::new(u64::MAX),
        singleton_class: RefCell::new(None),
        ivar: RefCell::new(RHashMap::default()),
    })
}

// ---------------------------------------------------------------------------
// Class methods
// ---------------------------------------------------------------------------

/// Time.now
/// Calls Time.__source to get [sec, nsec], then creates a Time object.
/// utc_offset defaults to 0 (UTC).
fn mrb_time_now(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let time_class_obj = vm
        .get_const_by_name("Time")
        .ok_or_else(|| Error::RuntimeError("Time class not found".to_string()))?;

    // Call Time.__source -> [sec, nsec]
    let source = mrb_funcall(vm, Some(time_class_obj), "__source", &[])?;
    let (sec, nsec) = source.as_ref().try_into()?;

    Ok(make_time_object(vm, RTimeData::new(sec, nsec, 0)))
}

/// Time.at(sec) or Time.at(sec, nsec)
fn mrb_time_at(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = strip_trailing_nil(args);
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1+)".to_string(),
        ));
    }

    let sec = get_integer_or_float_as_i64(&args[0])?;
    let nsec = if args.len() >= 2 {
        get_integer_or_float_as_u32(&args[1])?
    } else {
        0
    };

    Ok(make_time_object(vm, RTimeData::new(sec, nsec, 0)))
}

// ---------------------------------------------------------------------------
// Instance methods
// ---------------------------------------------------------------------------

/// Time#year
fn mrb_time_year(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let (year, _, _, _, _, _, _) = t.to_datetime_parts();
    Ok(RObject::integer(year as i64).to_refcount_assigned())
}

/// Time#month (alias: mon)
fn mrb_time_month(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let (_, month, _, _, _, _, _) = t.to_datetime_parts();
    Ok(RObject::integer(month as i64).to_refcount_assigned())
}

/// Time#day (alias: mday)
fn mrb_time_day(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let (_, _, day, _, _, _, _) = t.to_datetime_parts();
    Ok(RObject::integer(day as i64).to_refcount_assigned())
}

/// Time#wday (0=Sunday, 1=Monday, ..., 6=Saturday)
fn mrb_time_wday(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let (_, _, _, wday, _, _, _) = t.to_datetime_parts();
    Ok(RObject::integer(wday as i64).to_refcount_assigned())
}

/// Time#hour
fn mrb_time_hour(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let (_, _, _, _, hour, _, _) = t.to_datetime_parts();
    Ok(RObject::integer(hour as i64).to_refcount_assigned())
}

/// Time#min
fn mrb_time_min(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let (_, _, _, _, _, min, _) = t.to_datetime_parts();
    Ok(RObject::integer(min as i64).to_refcount_assigned())
}

/// Time#sec
fn mrb_time_sec(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let (_, _, _, _, _, _, sec) = t.to_datetime_parts();
    Ok(RObject::integer(sec as i64).to_refcount_assigned())
}

/// Time#nsec
fn mrb_time_nsec(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    Ok(RObject::integer(t.nsec as i64).to_refcount_assigned())
}

/// Time#to_s -> "%Y-%m-%d %H:%M:%S %z"
fn mrb_time_to_s(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    Ok(RObject::string(t.to_s()).to_refcount_assigned())
}

/// Time#+ (sec as integer or float)
fn mrb_time_add(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = strip_trailing_nil(args);
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;

    let (delta_sec, delta_nsec) = float_to_sec_nsec(&args[0])?;
    let new_nsec = t.nsec as i64 + delta_nsec as i64;
    let carry = new_nsec.div_euclid(1_000_000_000);
    let new_nsec = new_nsec.rem_euclid(1_000_000_000) as u32;
    let new_sec = t.sec + delta_sec + carry;

    Ok(make_time_object(
        vm,
        RTimeData::new(new_sec, new_nsec, t.utc_offset),
    ))
}

/// Time#- (sec as integer or float), also supports Time - Time -> Float (seconds)
fn mrb_time_sub(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = strip_trailing_nil(args);
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;

    // Check if rhs is a Time object
    if let RValue::Data(_) = &args[0].value
        && let Ok(rhs) = get_time_data(&args[0])
    {
        // Time - Time -> Float (difference in seconds)
        let sec_diff =
            (t.sec - rhs.sec) as f64 + (t.nsec as f64 - rhs.nsec as f64) / 1_000_000_000.0;
        return Ok(RObject::float(sec_diff).to_refcount_assigned());
    }

    let (delta_sec, delta_nsec) = float_to_sec_nsec(&args[0])?;
    let new_nsec = t.nsec as i64 - delta_nsec as i64;
    let carry = new_nsec.div_euclid(1_000_000_000);
    let new_nsec = new_nsec.rem_euclid(1_000_000_000) as u32;
    let new_sec = t.sec - delta_sec + carry;

    Ok(make_time_object(
        vm,
        RTimeData::new(new_sec, new_nsec, t.utc_offset),
    ))
}

/// Time#<=> (compare with another Time object)
fn mrb_time_cmp(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = strip_trailing_nil(args);
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;

    let rhs = match get_time_data(&args[0]) {
        Ok(r) => r,
        Err(_) => return Ok(RObject::nil().to_refcount_assigned()),
    };

    let result = match t.sec.cmp(&rhs.sec) {
        std::cmp::Ordering::Equal => t.nsec.cmp(&rhs.nsec),
        other => other,
    };

    let int_val = match result {
        std::cmp::Ordering::Less => -1i64,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    Ok(RObject::integer(int_val).to_refcount_assigned())
}

/// Time#utc_offset -> Integer (seconds)
fn mrb_time_utc_offset(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    Ok(RObject::integer(t.utc_offset as i64).to_refcount_assigned())
}

/// Time#localtime(offset) - returns a new Time with the given UTC offset (in seconds)
fn mrb_time_localtime(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = strip_trailing_nil(args);
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;

    let new_offset = if args.is_empty() {
        0i32 // default to UTC if no arg
    } else {
        get_integer_or_float_as_i64(&args[0])? as i32
    };

    Ok(make_time_object(
        vm,
        RTimeData::new(t.sec, t.nsec, new_offset),
    ))
}

/// Time#to_i -> sec
fn mrb_time_to_i(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    Ok(RObject::integer(t.sec).to_refcount_assigned())
}

/// Time#to_f -> sec.nsec as float
fn mrb_time_to_f(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let t = get_time_data(&self_obj)?;
    let f = t.sec as f64 + t.nsec as f64 / 1_000_000_000.0;
    Ok(RObject::float(f).to_refcount_assigned())
}

// ---------------------------------------------------------------------------
// Default Time.__source (std::time based, non-wasm)
// ---------------------------------------------------------------------------

/// Default implementation of Time.__source using std::time.
/// Returns [sec, nsec] as a Ruby array.
/// Only compiled on non-wasm targets (wasi feature disabled implies std::time available).
#[cfg(not(target_arch = "wasm32"))]
fn mrb_time_source_default(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let unixtime = now
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::RuntimeError("system time before UNIX EPOCH".to_string()))?;
    let sec = unixtime.as_secs() as i64;
    let nsec = unixtime.subsec_nanos() as i64;
    let arr = vec![
        RObject::integer(sec).to_refcount_assigned(),
        RObject::integer(nsec).to_refcount_assigned(),
    ];
    Ok(RObject::array(arr).to_refcount_assigned())
}

// ---------------------------------------------------------------------------
// Helper utilities
// ---------------------------------------------------------------------------

fn strip_trailing_nil(args: &[Rc<RObject>]) -> &[Rc<RObject>] {
    if !args.is_empty() && args[args.len() - 1].is_nil() {
        &args[0..args.len() - 1]
    } else {
        args
    }
}

fn get_integer_or_float_as_i64(obj: &RObject) -> Result<i64, Error> {
    match &obj.value {
        RValue::Integer(i) => Ok(*i),
        RValue::Float(f) => Ok(*f as i64),
        _ => Err(Error::ArgumentError(
            "expected Integer or Float".to_string(),
        )),
    }
}

fn get_integer_or_float_as_u32(obj: &RObject) -> Result<u32, Error> {
    match &obj.value {
        RValue::Integer(i) => {
            if *i < 0 {
                return Err(Error::ArgumentError(
                    "nsec must be non-negative".to_string(),
                ));
            }
            Ok(*i as u32)
        }
        RValue::Float(f) => {
            if *f < 0.0 {
                return Err(Error::ArgumentError(
                    "nsec must be non-negative".to_string(),
                ));
            }
            Ok(*f as u32)
        }
        _ => Err(Error::ArgumentError(
            "expected Integer or Float".to_string(),
        )),
    }
}

/// Convert a numeric seconds value (possibly fractional) to (whole_sec, nsec).
fn float_to_sec_nsec(obj: &RObject) -> Result<(i64, u32), Error> {
    match &obj.value {
        RValue::Integer(i) => Ok((*i, 0)),
        RValue::Float(f) => {
            let sec = f.trunc() as i64;
            let nsec = (f.fract().abs() * 1_000_000_000.0).round() as u32;
            Ok((sec, nsec))
        }
        _ => Err(Error::ArgumentError(
            "expected Integer or Float".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Public initializer
// ---------------------------------------------------------------------------

/// Initialize the Time class in the VM.
/// Call this after `VM::open` to make `Time` available in Ruby code.
pub fn init_time(vm: &mut VM) {
    let time_class = vm.define_class("Time", None, None);

    // Class methods
    mrb_define_class_cmethod(vm, time_class.clone(), "now", Box::new(mrb_time_now));
    mrb_define_class_cmethod(vm, time_class.clone(), "at", Box::new(mrb_time_at));

    // Instance methods
    mrb_define_cmethod(vm, time_class.clone(), "year", Box::new(mrb_time_year));
    mrb_define_cmethod(vm, time_class.clone(), "month", Box::new(mrb_time_month));
    mrb_define_cmethod(vm, time_class.clone(), "mon", Box::new(mrb_time_month));
    mrb_define_cmethod(vm, time_class.clone(), "day", Box::new(mrb_time_day));
    mrb_define_cmethod(vm, time_class.clone(), "mday", Box::new(mrb_time_day));
    mrb_define_cmethod(vm, time_class.clone(), "wday", Box::new(mrb_time_wday));
    mrb_define_cmethod(vm, time_class.clone(), "hour", Box::new(mrb_time_hour));
    mrb_define_cmethod(vm, time_class.clone(), "min", Box::new(mrb_time_min));
    mrb_define_cmethod(vm, time_class.clone(), "sec", Box::new(mrb_time_sec));
    mrb_define_cmethod(vm, time_class.clone(), "nsec", Box::new(mrb_time_nsec));
    mrb_define_cmethod(vm, time_class.clone(), "to_s", Box::new(mrb_time_to_s));
    mrb_define_cmethod(vm, time_class.clone(), "inspect", Box::new(mrb_time_to_s));
    mrb_define_cmethod(vm, time_class.clone(), "+", Box::new(mrb_time_add));
    mrb_define_cmethod(vm, time_class.clone(), "-", Box::new(mrb_time_sub));
    mrb_define_cmethod(vm, time_class.clone(), "<=>", Box::new(mrb_time_cmp));
    mrb_define_cmethod(
        vm,
        time_class.clone(),
        "utc_offset",
        Box::new(mrb_time_utc_offset),
    );
    mrb_define_cmethod(
        vm,
        time_class.clone(),
        "gmt_offset",
        Box::new(mrb_time_utc_offset),
    );
    mrb_define_cmethod(
        vm,
        time_class.clone(),
        "localtime",
        Box::new(mrb_time_localtime),
    );
    mrb_define_cmethod(vm, time_class.clone(), "to_i", Box::new(mrb_time_to_i));
    mrb_define_cmethod(vm, time_class.clone(), "to_f", Box::new(mrb_time_to_f));

    // Register default Time.__source on non-wasm targets
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _time_class_obj = RObject::class(time_class, vm);
        let time_class_obj_for_source = vm
            .get_const_by_name("Time")
            .expect("Time class not found after definition");
        mrb_define_class_cmethod_on_obj(
            vm,
            time_class_obj_for_source,
            "__source",
            Box::new(mrb_time_source_default),
        );
    }
}

/// Helper: define a singleton (class-side) cmethod on a class RObject.
#[cfg(not(target_arch = "wasm32"))]
fn mrb_define_class_cmethod_on_obj(
    vm: &mut VM,
    class_obj: Rc<RObject>,
    name: &str,
    cmethod: mrubyedge::yamrb::value::RFn,
) {
    use mrubyedge::yamrb::helpers::mrb_define_singleton_cmethod;
    mrb_define_singleton_cmethod(vm, class_obj, name, cmethod);
}
