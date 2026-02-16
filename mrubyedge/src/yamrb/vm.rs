use std::cell::{Cell, RefCell};
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::{array, env};

use crate::Error;
use crate::rite::{Irep, Rite, insn};

use super::op::Op;
use super::prelude::prelude;
use super::value::RHashMap;
use super::value::*;
use super::{op, optable::*};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENGINE: &str = "mruby/edge";

const MAX_REGS_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub enum TargetContext {
    Class(Rc<RClass>),
    Module(Rc<RModule>),
}

impl TargetContext {
    pub fn name(&self) -> String {
        match self {
            TargetContext::Class(c) => c.full_name(),
            TargetContext::Module(m) => m.full_name(),
        }
    }
}

#[derive(Debug)]
pub struct Breadcrumb {
    pub event: &'static str, // TODO: be enum
    pub caller: Option<String>,
    pub return_reg: Option<usize>,
    pub upper: Option<Rc<Breadcrumb>>,
}

#[derive(Debug)]
pub struct KArgs {
    pub args: RefCell<RHashMap<RSym, Rc<RObject>>>,
    pub kwrest_reg: Cell<usize>,
    pub upper: Option<Rc<KArgs>>,
}

impl Breadcrumb {
    #[cfg(feature = "mrubyedge-debug")]
    pub fn display_breadcrumb_for_debug(&self, level: usize, max_level: usize) -> bool {
        if level > max_level {
            return false;
        }
        eprintln!(
            "{}- Breadcrumb: event='{}', caller={}, return_reg={:?}",
            "  ".repeat(level),
            self.event,
            self.caller.as_deref().unwrap_or("(none)"),
            self.return_reg
        );
        if let Some(upper) = &self.upper {
            upper.display_breadcrumb_for_debug(level + 1, max_level);
        }
        true
    }
}

pub struct VM {
    pub irep: Rc<IREP>,

    pub id: usize,
    pub bytecode: Vec<u8>,
    pub current_irep: Rc<IREP>,
    pub pc: Cell<usize>,
    pub regs: [Option<Rc<RObject>>; MAX_REGS_SIZE],
    pub current_regs_offset: usize,
    pub current_callinfo: Option<Rc<CALLINFO>>,
    pub current_breadcrumb: Option<Rc<Breadcrumb>>,
    pub kargs: RefCell<Option<RHashMap<RSym, Rc<RObject>>>>,
    pub current_kargs: RefCell<Option<Rc<KArgs>>>,
    pub target_class: TargetContext,
    pub exception: Option<Rc<RException>>,

    pub flag_preemption: Cell<bool>,

    #[cfg(feature = "insn-limit")]
    pub insn_count: Cell<usize>,
    #[cfg(feature = "insn-limit")]
    pub insn_limit: usize,

    // common class
    pub object_class: Rc<RClass>,
    pub builtin_class_table: RHashMap<&'static str, Rc<RClass>>,
    pub class_object_table: RHashMap<String, Rc<RObject>>,

    pub globals: RHashMap<String, Rc<RObject>>,
    pub consts: RHashMap<String, Rc<RObject>>,

    pub upper: Option<Rc<ENV>>,
    // TODO: using fixed array?
    pub cur_env: RHashMap<usize, Rc<ENV>>,
    pub has_env_ref: RHashMap<usize, bool>,

    pub fn_table: RFnTable,
    pub fn_block_stack: RFnStack,
}

pub struct RFnTable {
    pub size: Cell<usize>,
    pub table: [MaybeUninit<Rc<RFn>>; 4096],
}

impl RFnTable {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            size: Cell::new(0),
            table: array::from_fn(|_| MaybeUninit::uninit()),
        }
    }

    pub fn set(&mut self, f: Rc<RFn>) {
        let i = self.size.get();
        if i >= self.table.len() {
            panic!("RFnTable overflow");
        }

        self.table[i].write(f);
        let size = self.size.get();
        if i >= size {
            self.size.set(i + 1);
        }
    }

    pub fn get(&self, i: usize) -> Option<Rc<RFn>> {
        if i >= self.size.get() {
            return None;
        }

        unsafe { self.table[i].assume_init_ref() }.clone().into()
    }

    pub fn len(&self) -> usize {
        self.size.get()
    }

    pub fn is_empty(&self) -> bool {
        self.size.get() == 0
    }
}

pub struct RFnStack {
    pub size: Cell<usize>,
    pub stack: [Option<Rc<RFn>>; 64],
}

impl RFnStack {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            size: Cell::new(0),
            stack: array::from_fn(|_| None),
        }
    }

    pub fn push(&mut self, f: Rc<RFn>) -> Result<(), Error> {
        let i = self.size.get();
        if i >= self.stack.len() {
            return Err(Error::internal("RFnStack overflow"));
        }

        self.stack[i] = Some(f);
        let size = self.size.get();
        if i >= size {
            self.size.set(i + 1);
        }
        Ok(())
    }

    pub fn pop(&self) -> Result<Rc<RFn>, Error> {
        let i = self.size.get();
        if i == 0 {
            return Err(Error::internal("RFnStack underflow"));
        }

        self.size.set(i - 1);

        self.stack[i - 1]
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::internal("RFnStack invalid state"))
    }

    pub fn len(&self) -> usize {
        self.size.get()
    }

    pub fn is_empty(&self) -> bool {
        self.size.get() == 0
    }
}

impl VM {
    /// Builds a VM from a parsed Rite chunk, consuming the bytecode and
    /// preparing the VM so it can be executed via [`VM::run`].
    pub fn open(rite: &mut Rite) -> VM {
        let irep = rite_to_irep(rite);

        VM::new_by_raw_irep(irep)
    }

    /// Returns a VM backed by an empty IREP that immediately executes a
    /// `STOP` instruction. Useful for tests or placeholder VMs.
    pub fn empty() -> VM {
        let irep = IREP {
            __id: 0,
            nlocals: 0,
            nregs: 0,
            rlen: 0,
            code: vec![op::Op {
                code: insn::OpCode::STOP,
                operand: insn::Fetched::Z,
                pos: 18,
                len: 1,
            }],
            syms: Vec::new(),
            pool: Vec::new(),
            reps: Vec::new(),
            lv: None,
            catch_target_pos: Vec::new(),
        };
        Self::new_by_raw_irep(irep)
    }

    /// Creates a VM directly from a raw [`IREP`] tree without going through the
    /// Rite loader. This wires up the register file, globals, and builtin
    /// tables and runs the prelude to seed standard classes.
    pub fn new_by_raw_irep(irep: IREP) -> VM {
        let irep = Rc::new(irep);
        let globals = RHashMap::default();
        let consts = RHashMap::default();
        let builtin_class_table = RHashMap::default();
        let class_object_table = RHashMap::default();

        let object_class = Rc::new(RClass::new("Object", None, None));
        object_class.update_module_weakref();

        let id = 1; // TODO generator
        let bytecode = Vec::new();
        let current_irep = irep.clone();
        let pc = Cell::new(0);
        let regs: [Option<Rc<RObject>>; MAX_REGS_SIZE] = [const { None }; MAX_REGS_SIZE];
        let current_regs_offset = 0;
        let current_callinfo = None;
        let current_breadcrumb = Some(Rc::new(Breadcrumb {
            upper: None,
            event: "root",
            caller: None,
            return_reg: None,
        }));
        let kargs = RefCell::new(None);
        let current_kargs = RefCell::new(None);
        let target_class = TargetContext::Class(object_class.clone());
        let exception = None;
        let flag_preemption = Cell::new(false);
        let fn_table = RFnTable::new();
        let fn_block_stack = RFnStack::new();
        let upper = None;
        let cur_env = RHashMap::default();
        let has_env_ref = RHashMap::default();

        #[cfg(feature = "insn-limit")]
        let insn_count = Cell::new(0);
        #[cfg(feature = "insn-limit")]
        let insn_limit = {
            let limit_str = env!("MRUBYEDGE_INSN_LIMIT", "MRUBYEDGE_INSN_LIMIT must be set when insn-limit feature is enabled");
            limit_str.parse::<usize>().expect("MRUBYEDGE_INSN_LIMIT must be a valid number")
        };

        let mut vm = VM {
            id,
            bytecode,
            irep,
            current_irep,
            pc,
            regs,
            current_regs_offset,
            current_callinfo,
            current_breadcrumb,
            kargs,
            current_kargs,
            target_class,
            exception,
            flag_preemption,
            #[cfg(feature = "insn-limit")]
            insn_count,
            #[cfg(feature = "insn-limit")]
            insn_limit,
            object_class,
            builtin_class_table,
            class_object_table,
            globals,
            consts,
            upper,
            cur_env,
            has_env_ref,
            fn_table,
            fn_block_stack,
        };

        prelude(&mut vm);

        vm
    }

    /// Resets the instruction counter. Only available when the `insn-limit` feature is enabled.
    #[cfg(feature = "insn-limit")]
    pub fn reset_insn_count(&mut self) {
        self.insn_count.set(0);
    }

    /// Returns the current instruction count. Only available when the `insn-limit` feature is enabled.
    #[cfg(feature = "insn-limit")]
    pub fn get_insn_count(&self) -> usize {
        self.insn_count.get()
    }

    /// Executes the current IREP until completion, returning the value in
    /// register 0 or propagating any raised exception as an error. The
    /// top-level `self` is initialized automatically before evaluation.
    pub fn run(&mut self) -> Result<Rc<RObject>, Box<dyn std::error::Error>> {
        self.current_irep = self.irep.clone();
        self.pc.set(0);

        let upper = self.current_breadcrumb.take();
        let new_breadcrumb = Rc::new(Breadcrumb {
            upper,
            event: "run",
            caller: None,
            return_reg: None,
        });
        self.current_breadcrumb.replace(new_breadcrumb);
        self.__run()
    }

    /// Internal run method that manages breadcrumb stack for internal calls.
    pub fn run_internal(&mut self) -> Result<Rc<RObject>, Box<dyn std::error::Error>> {
        let upper = self.current_breadcrumb.take();
        let new_breadcrumb = Rc::new(Breadcrumb {
            upper,
            event: "run_internal",
            caller: None,
            return_reg: None,
        });
        self.current_breadcrumb.replace(new_breadcrumb);
        self.__run()
    }

    pub fn eval_rite(
        &mut self,
        rite: &mut Rite,
    ) -> Result<Rc<RObject>, Box<dyn std::error::Error>> {
        let irep = rite_to_irep(rite);
        self.pc.set(0);
        self.current_irep = Rc::new(irep);

        let upper = self.current_breadcrumb.take();
        let new_breadcrumb = Rc::new(Breadcrumb {
            upper,
            event: "eval",
            caller: None,
            return_reg: None,
        });
        self.current_breadcrumb.replace(new_breadcrumb);
        self.__run()
    }

    fn __run(&mut self) -> Result<Rc<RObject>, Box<dyn std::error::Error>> {
        let class = self.object_class.clone();
        // Insert top_self
        let top_self = RObject {
            tt: RType::Instance,
            value: RValue::Instance(RInstance {
                class,
                ref_count: 1,
            }),
            object_id: 0.into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
        .to_refcount_assigned();
        if self.current_regs()[0].is_none() {
            self.current_regs()[0].replace(top_self.clone());
        }
        let mut rescued = false;

        loop {
            if !rescued && let Some(e) = self.exception.clone() {
                let operand = insn::Fetched::B(0);
                let mut retreg = None;
                if let Some(pos) = self.find_next_handler_pos() {
                    self.pc.set(pos);
                    rescued = true;
                    continue;
                }

                if let Error::BlockReturn(id, v) = e.error_type.borrow().clone()
                    && self.current_irep.__id == id
                {
                    // reached caller method's IREP, just return
                    let operand = insn::Fetched::B(16); // FIXME: just a bit far reg
                    self.current_regs()[16].replace(v);
                    self.exception.take();
                    op_return(self, &operand).expect("[bug]cannot return");
                    continue;
                }

                if matches!(e.error_type.borrow().clone(), Error::Break(_)) {
                    retreg = match self.current_breadcrumb.as_ref() {
                        Some(bc) if bc.event == "do_op_send" => {
                            let retreg = bc.as_ref().return_reg.unwrap_or(0);
                            Some(retreg)
                        }
                        _ => None,
                    };
                }
                match op_return(self, &operand) {
                    Ok(_) => {}
                    Err(_) => {
                        if let Some(retreg) = retreg
                            && let Error::Break(brkval) = e.error_type.borrow().clone()
                        {
                            self.current_regs()[retreg].replace(brkval);
                            self.exception.take();
                        } else {
                            break;
                        }
                    }
                }
                if self.flag_preemption.get() {
                    break;
                } else {
                    continue;
                }
            }
            rescued = false;

            let pc = self.pc.get();
            if self.current_irep.code.len() <= pc {
                // reached end of the IREP
                break;
            }
            let op = *self
                .current_irep
                .code
                .get(pc)
                .ok_or_else(|| Error::internal("end of opcode reached"))?;
            let operand = op.operand;
            self.pc.set(pc + 1);

            #[cfg(feature = "insn-limit")]
            {
                let count = self.insn_count.get();
                if count >= self.insn_limit {
                    return Err(Box::new(Error::internal(format!(
                        "instruction limit exceeded: {} instructions",
                        self.insn_limit
                    ))));
                }
                self.insn_count.set(count + 1);
            }

            #[cfg(feature = "mrubyedge-debug")]
            if let Ok(v) = env::var("MRUBYEDGE_DEBUG") {
                let level: i32 = v.parse().unwrap_or(1);
                if level >= 2 {
                    self.debug_dump_to_stdout(32);
                }
                eprintln!(
                    "{:?}: {:?} (pos={} len={})",
                    op.code, &operand, op.pos, op.len
                );
            }

            match consume_expr(self, op.code, &operand, op.pos, op.len) {
                Ok(_) => {}
                Err(e) => {
                    let exception = RException::from_error(self, &e);
                    self.exception = Some(Rc::new(exception));
                    continue;
                }
            }

            if self.flag_preemption.get() {
                break;
            }
        }

        self.flag_preemption.set(false);

        if let Some(e) = self.exception.clone() {
            return Err(e.error_type.borrow().clone().into());
        }

        let retval = match self.current_regs()[0].take() {
            Some(v) => Ok(v),
            None => Ok(Rc::new(RObject::nil())),
        };
        self.current_regs()[0].replace(top_self.clone());

        retval
    }

    pub(crate) fn find_next_handler_pos(&mut self) -> Option<usize> {
        let ci = self.pc.get();
        for p in self.current_irep.catch_target_pos.iter() {
            if ci < *p {
                return Some(*p);
            }
        }
        None
    }

    pub(crate) fn current_regs(&mut self) -> &mut [Option<Rc<RObject>>] {
        &mut self.regs[self.current_regs_offset..]
    }

    pub(crate) fn get_current_regs_cloned(&mut self, i: usize) -> Result<Rc<RObject>, Error> {
        self.current_regs()[i]
            .clone()
            .ok_or_else(|| Error::internal(format!("register {} is not assigned", i)))
    }

    pub(crate) fn take_current_regs(&mut self, i: usize) -> Result<Rc<RObject>, Error> {
        self.current_regs()[i]
            .take()
            .ok_or_else(|| Error::internal(format!("register {} is not assigned", i)))
    }

    /// Returns the current `self` object from register 0, or an error if it has
    /// not been initialized yet.
    pub fn getself(&mut self) -> Result<Rc<RObject>, Error> {
        self.get_current_regs_cloned(0)
    }

    /// Retrieves `self` without error handling, panicking if register 0 is
    /// empty. Prefer [`VM::getself`] when the value may be absent.
    pub fn must_getself(&mut self) -> Rc<RObject> {
        self.current_regs()[0]
            .clone()
            .expect("self is not assigned")
    }

    pub fn get_kwargs(&self) -> Option<RHashMap<String, Rc<RObject>>> {
        let kwargs = self.current_kargs.borrow().clone();
        kwargs.map(|kargs| {
            kargs
                .args
                .borrow()
                .iter()
                .map(|(k, v)| (k.name.clone(), v.clone()))
                .collect()
        })
    }

    pub(crate) fn register_fn(&mut self, f: RFn) -> usize {
        self.fn_table.set(Rc::new(f));
        self.fn_table.len() - 1
    }

    pub(crate) fn push_fnblock(&mut self, f: Rc<RFn>) -> Result<(), Error> {
        self.fn_block_stack.push(f)
    }

    pub(crate) fn pop_fnblock(&mut self) -> Result<Rc<RFn>, Error> {
        self.fn_block_stack.pop()
    }

    pub(crate) fn get_fn(&self, i: usize) -> Option<Rc<RFn>> {
        self.fn_table.get(i)
    }

    /// Looks up a previously defined builtin class by name. Panics if the
    /// class does not exist, which usually signals a missing prelude setup.
    pub fn get_class_by_name(&self, name: &str) -> Rc<RClass> {
        self.builtin_class_table
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Class {} not found", name))
    }

    pub fn get_module_by_name(&self, name: &str) -> Rc<RModule> {
        match self.consts.get(name).cloned() {
            Some(obj) => match &obj.value {
                RValue::Module(m) => m.clone(),
                _ => panic!("Module {} not found", name),
            },
            None => panic!("Module {} not found", name),
        }
    }

    pub fn get_const_by_name(&self, name: &str) -> Option<Rc<RObject>> {
        self.consts.get(name).cloned()
    }

    /// Defines a new class under the optional parent module, inheriting from
    /// `superclass` or `Object` by default, and registers it in the constant
    /// table. The resulting class object is returned for further mutation.
    pub fn define_class(
        &mut self,
        name: &str,
        superclass: Option<Rc<RClass>>,
        parent_module: Option<Rc<RModule>>,
    ) -> Rc<RClass> {
        let superclass = match superclass {
            Some(c) => c,
            None => self.object_class.clone(),
        };
        let class = Rc::new(RClass::new(name, Some(superclass), parent_module.clone()));
        class.update_module_weakref();

        let object = RObject::class(class.clone(), self);
        self.consts.insert(name.to_string(), object.clone());
        if let Some(parent) = parent_module {
            parent
                .consts
                .borrow_mut()
                .insert(name.to_string(), object.clone());
        } else {
            self.object_class
                .consts
                .borrow_mut()
                .insert(name.to_string(), object);
        }
        class
    }

    /// Defines a new module, optionally nested under another module, and stores
    /// it in the VM's constant table so it becomes accessible to Ruby code.
    pub fn define_module(&mut self, name: &str, parent_module: Option<Rc<RModule>>) -> Rc<RModule> {
        let module = Rc::new(RModule::new(name));
        if let Some(parent) = parent_module {
            module.parent.replace(Some(parent));
        }
        let object = RObject::module(module.clone()).to_refcount_assigned();
        self.consts.insert(name.to_string(), object.clone());
        self.object_class
            .consts
            .borrow_mut()
            .insert(name.to_string(), object);
        module
    }

    pub(crate) fn define_standard_class(&mut self, name: &'static str) -> Rc<RClass> {
        let class = self.define_class(name, None, None);
        self.builtin_class_table.insert(name, class.clone());
        class
    }

    pub(crate) fn define_standard_class_with_superclass(
        &mut self,
        name: &'static str,
        superclass: Rc<RClass>,
    ) -> Rc<RClass> {
        let class = self.define_class(name, Some(superclass.clone()), None);
        self.builtin_class_table.insert(name, class.clone());
        class
    }

    #[allow(dead_code)]
    pub(crate) fn define_standard_class_under(
        &mut self,
        name: &'static str,
        parent: Rc<RModule>,
    ) -> Rc<RClass> {
        let class = self.define_class(name, None, Some(parent));
        self.builtin_class_table.insert(name, class.clone());
        class
    }

    #[allow(dead_code)]
    pub(crate) fn define_standard_class_with_superclass_under(
        &mut self,
        name: &'static str,
        superclass: Rc<RClass>,
        parent: Rc<RModule>,
    ) -> Rc<RClass> {
        let class = self.define_class(name, Some(superclass.clone()), Some(parent));
        self.builtin_class_table.insert(name, class.clone());
        class
    }

    #[allow(unused)]
    pub fn debug_dump_to_stdout(&mut self, max_breadcrumb_level: usize) {
        #[cfg(feature = "mrubyedge-debug")]
        {
            use crate::yamrb::helpers::mrb_call_inspect;
            eprintln!("=== VM Dump ===");
            eprintln!("ID: {}", self.id);
            eprintln!("Current IREP ID: {}", self.current_irep.__id);
            eprintln!("PC: {}", self.pc.get());
            let current_regs_offset = self.current_regs_offset;
            eprintln!("IREPs:");
            self.current_irep
                .code
                .iter()
                .enumerate()
                .for_each(|(i, op)| {
                    eprintln!("{:04} {:?}: {:?}", i, op.code, op.operand);
                });
            eprintln!("Current Regs Offset: {}", current_regs_offset);
            eprintln!("Regs:");
            let size = self.regs.len();
            for i in 0..size {
                let reg = self.regs.get(i).unwrap().clone();
                if let Some(obj) = reg {
                    let inspect: String = mrb_call_inspect(self, obj.clone())
                        .unwrap()
                        .as_ref()
                        .try_into()
                        .unwrap_or_else(|_| "(uninspectable)".into());
                    if i < current_regs_offset {
                        eprintln!("  R{}(--): {}(oid={})", i, inspect, obj.object_id.get());
                    } else {
                        eprintln!(
                            "  R{}(R{}): {}(oid={})",
                            i,
                            i - current_regs_offset,
                            inspect,
                            obj.object_id.get()
                        );
                    }
                } else if i < 16 || i < current_regs_offset {
                    eprintln!("  R{}(--): <None>", i);
                } else {
                    break;
                }
            }
            // eprintln!("Current CallInfo: {:?}", self.current_callinfo);
            eprintln!("Target Class: {}", self.target_class.name());
            eprintln!(
                "Exception: {:?}",
                self.exception
                    .as_deref()
                    .map(|e| e.error_type.borrow().clone())
            );
            eprintln!("--- Breadcrumb ---");
            if let Some(bc) = &self.current_breadcrumb {
                bc.display_breadcrumb_for_debug(0, max_breadcrumb_level);
            } else {
                eprintln!("(none)");
            }
            eprintln!("=== End of VM Dump ===");
        }
    }

    pub fn get_outermost_env(&self) -> Option<Rc<ENV>> {
        let mut env = self.upper.clone();
        while let Some(e) = env.clone() {
            if e.upper.is_none() {
                return env;
            }
            env = e.upper.clone();
        }
        env
    }
}

fn interpret_insn(mut insns: &[u8]) -> Vec<Op> {
    let mut pos: usize = 0;
    let mut ops = Vec::new();
    while !insns.is_empty() {
        let op = insns[0];
        let opcode: insn::OpCode = op.try_into().unwrap();
        let fetched = insn::FETCH_TABLE[op as usize](&mut insns).unwrap();
        ops.push(Op::new(opcode, fetched, pos, 1 + fetched.len()));
        pos += 1 + fetched.len();
    }
    ops
}

fn load_irep_1(reps: &mut [Irep], pos: usize) -> (IREP, usize) {
    let irep = &mut reps[pos];
    let mut irep1 = IREP {
        __id: pos,
        nlocals: irep.nlocals(),
        nregs: irep.nregs(),
        rlen: irep.rlen(),
        code: Vec::new(),
        syms: Vec::new(),
        pool: Vec::new(),
        reps: Vec::new(),
        lv: None,
        catch_target_pos: Vec::new(),
    };
    for sym in irep.syms.iter() {
        irep1
            .syms
            .push(RSym::new(sym.to_string_lossy().to_string()));
    }
    for val in irep.pool.iter() {
        match val {
            crate::rite::PoolValue::Str(s) | crate::rite::PoolValue::SStr(s) => {
                irep1.pool.push(RPool::Str(s.to_string_lossy().to_string()));
            }
            crate::rite::PoolValue::Int32(i) => {
                irep1.pool.push(RPool::Int(*i as i64));
            }
            crate::rite::PoolValue::Int64(i) => {
                irep1.pool.push(RPool::Int(*i));
            }
            crate::rite::PoolValue::Float(f) => {
                irep1.pool.push(RPool::Float(*f));
            }
            crate::rite::PoolValue::BigInt(_) => {
                // BigInt not yet supported, store as 0 for now
                irep1.pool.push(RPool::Int(0));
            }
        }
    }
    let code = interpret_insn(irep.insn);
    for ch in irep.catch_handlers.iter() {
        let pos = ch.target;
        let (i, _) = code
            .iter()
            .enumerate()
            .find(|(_, op)| op.pos == pos)
            .expect("catch handler mismatch");
        irep1.catch_target_pos.push(i);
    }
    let mut map = RHashMap::default();
    for (reg, name) in irep.lv.iter().enumerate() {
        if let Some(name) = name {
            // lv register index in mruby is 1-based
            map.insert(reg + 1, name.to_string_lossy().to_string());
        }
    }
    if !map.is_empty() {
        irep1.lv = Some(map);
    }
    irep1.catch_target_pos.sort();

    irep1.code = code;
    (irep1, pos + 1)
}

fn load_irep_0(reps: &mut [Irep], pos: usize) -> (IREP, usize) {
    let (mut irep0, newpos) = load_irep_1(reps, pos);
    let mut pos = newpos;
    for _ in 0..irep0.rlen {
        let (rep, newpos) = load_irep_0(reps, pos);
        pos = newpos;
        irep0.reps.push(Rc::new(rep));
    }
    (irep0, pos)
}

// This will consume the Rite object and return the IREP
fn rite_to_irep(rite: &mut Rite) -> IREP {
    let (irep0, _) = load_irep_0(&mut rite.irep, 0);
    irep0
}

#[derive(Debug, Clone)]
pub struct IREP {
    pub __id: usize,

    pub nlocals: usize,
    pub nregs: usize, // NOTE: is u8 better?
    pub rlen: usize,
    pub code: Vec<Op>,
    pub syms: Vec<RSym>,
    pub pool: Vec<RPool>,
    pub reps: Vec<Rc<IREP>>,
    pub lv: Option<RHashMap<usize, String>>,
    pub catch_target_pos: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CALLINFO {
    pub prev: Option<Rc<CALLINFO>>,
    pub method_id: RSym,
    pub pc_irep: Rc<IREP>,
    pub pc: usize,
    pub current_regs_offset: usize,
    pub target_class: TargetContext,
    pub n_args: usize,
    pub return_reg: usize,
    pub method_owner: Option<Rc<RModule>>,
}

#[derive(Debug, Clone)]
pub struct ENV {
    pub __irep_id: usize,
    pub upper: Option<Rc<ENV>>,
    pub captured: RefCell<Option<Vec<Option<Rc<RObject>>>>>,
    pub current_regs_offset: usize,
    pub is_expired: Cell<bool>,
}

impl ENV {
    #[allow(unused)]
    pub(crate) fn has_captured(&self) -> bool {
        self.captured.borrow().is_some()
    }

    #[allow(unused)]
    pub(crate) fn capture(&self, regs: &[Option<Rc<RObject>>]) {
        let mut captured = self.captured.borrow_mut();
        captured.replace(regs.to_vec());
    }

    pub(crate) fn capture_no_clone(&self, regs: Vec<Option<Rc<RObject>>>) {
        let mut captured = self.captured.borrow_mut();
        captured.replace(regs);
    }

    pub(crate) fn expire(&self) {
        self.is_expired.set(true);
    }

    pub(crate) fn expired(&self) -> bool {
        self.is_expired.get()
    }
}
