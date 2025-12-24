use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::rc::Rc;

use crate::Error;
use crate::rite::{Irep, Rite, insn};
use crate::yamrb::helpers::mrb_call_inspect;

use super::op::Op;
use super::prelude::prelude;
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

impl Breadcrumb {
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
    pub target_class: TargetContext,
    pub exception: Option<Rc<RException>>,

    pub flag_preemption: Cell<bool>,

    // common class
    pub object_class: Rc<RClass>,
    pub builtin_class_table: HashMap<&'static str, Rc<RClass>>,
    pub class_object_table: HashMap<String, Rc<RObject>>,

    pub globals: HashMap<String, Rc<RObject>>,
    pub consts: HashMap<String, Rc<RObject>>,

    pub break_level: usize,
    pub break_value: RefCell<Option<Rc<RObject>>>,
    pub break_target_level: Cell<Option<usize>>,

    pub upper: Option<Rc<ENV>>,
    // TODO: using fixed array?
    pub cur_env: HashMap<usize, Rc<ENV>>,
    pub has_env_ref: HashMap<usize, bool>,

    pub fn_table: Vec<Rc<RFn>>,
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
            catch_target_pos: Vec::new(),
        };
        Self::new_by_raw_irep(irep)
    }

    /// Creates a VM directly from a raw [`IREP`] tree without going through the
    /// Rite loader. This wires up the register file, globals, and builtin
    /// tables and runs the prelude to seed standard classes.
    pub fn new_by_raw_irep(irep: IREP) -> VM {
        let irep = Rc::new(irep);
        let globals = HashMap::new();
        let consts = HashMap::new();
        let builtin_class_table = HashMap::new();
        let class_object_table = HashMap::new();

        let object_class = Rc::new(RClass::new("Object", None, None));

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
        let target_class = TargetContext::Class(object_class.clone());
        let exception = None;
        let flag_preemption = Cell::new(false);
        let fn_table = Vec::new();
        let break_level = 0;
        let break_value = RefCell::new(None);
        let break_target_level = Cell::new(None);
        let upper = None;
        let cur_env = HashMap::new();
        let has_env_ref = HashMap::new();

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
            target_class,
            exception,
            flag_preemption,
            object_class,
            builtin_class_table,
            class_object_table,
            globals,
            consts,
            break_level,
            break_value,
            break_target_level,
            upper,
            cur_env,
            has_env_ref,
            fn_table,
        };

        prelude(&mut vm);

        vm
    }

    /// Executes the current IREP until completion, returning the value in
    /// register 0 or propagating any raised exception as an error. The
    /// top-level `self` is initialized automatically before evaluation.
    pub fn run(&mut self) -> Result<Rc<RObject>, Box<dyn std::error::Error>> {
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
            ivar: RefCell::new(HashMap::new()),
        }
        .to_refcount_assigned();
        if self.current_regs()[0].is_none() {
            self.current_regs()[0].replace(top_self.clone());
        }
        let mut rescued = false;

        loop {
            if !rescued && let Some(e) = self.exception.clone() {
                let operand = insn::Fetched::B(0);
                if let Some(pos) = self.find_next_handler_pos() {
                    self.pc.set(pos);
                    rescued = true;
                    continue;
                }

                let retreg = match self.current_breadcrumb.as_ref() {
                    Some(bc) if bc.event == "do_op_send" => {
                        let retreg = bc.as_ref().return_reg.unwrap_or(0);
                        // dbg!(("return to reg {:?}", retreg));
                        Some(retreg)
                    }
                    _ => None,
                };
                match op_return(self, &operand) {
                    Ok(_) => {}
                    Err(_) => {
                        if let Some(retreg) = retreg
                            && let Error::Break(brkval) = e.error_type.borrow().clone()
                        {
                            dbg!("return brkval");
                            self.break_level -= 1; // handle as return
                            self.current_regs()[retreg].replace(brkval);
                            self.exception.take();

                            // self.break_level -= 1;
                        } else {
                            // if let Error::Break(brkval) = e.error_type.borrow().clone() {
                            //     // dbg!("set break val to VM");
                            //     self.break_value.borrow_mut().replace(brkval.clone());
                            // }
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
            let op = self
                .current_irep
                .code
                .get(pc)
                .ok_or_else(|| Error::internal("end of opcode reached"))?;
            let operand = op.operand;
            self.pc.set(pc + 1);

            if env::var("MRUBYEDGE_DEBUG").is_ok() {
                eprintln!(
                    "{:?}: {:?} (pos={} len={})",
                    &op.code, &op.operand, op.pos, op.len
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
            //dbg!(&self.current_breadcrumb);
            return Err(e.error_type.borrow().clone().into());
        }

        let retval = match self.current_regs()[0].take() {
            Some(v) => Ok(v),
            None => Ok(Rc::new(RObject::nil())),
        };
        self.current_regs()[0].replace(top_self.clone());

        //dbg!(&self.current_breadcrumb);
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

    pub(crate) fn register_fn(&mut self, f: RFn) -> usize {
        self.fn_table.push(Rc::new(f));
        self.fn_table.len() - 1
    }

    pub(crate) fn get_fn(&self, i: usize) -> Option<Rc<RFn>> {
        self.fn_table.get(i).cloned()
    }

    /// Looks up a previously defined builtin class by name. Panics if the
    /// class does not exist, which usually signals a missing prelude setup.
    pub fn get_class_by_name(&self, name: &str) -> Rc<RClass> {
        self.builtin_class_table
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Class {} not found", name))
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
        let class = Rc::new(RClass::new(name, Some(superclass), parent_module));
        let object = RObject::class(class.clone(), self);
        self.consts.insert(name.to_string(), object.clone());
        self.object_class
            .consts
            .borrow_mut()
            .insert(name.to_string(), object);
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

    pub fn debug_dump_to_stdout(&mut self, max_breadcrumb_level: usize) {
        eprintln!("=== VM Dump ===");
        eprintln!("ID: {}", self.id);
        eprintln!("PC: {}", self.pc.get());
        eprintln!("Current IREP ID: {}", self.current_irep.__id);
        eprintln!("Current Regs Offset: {}", self.current_regs_offset);
        eprintln!("Current Regs:");
        let size = self.current_regs().len();
        for i in 0..size {
            let reg = &self.get_current_regs_cloned(i).ok();
            if let Some(obj) = reg {
                let inspect: String = mrb_call_inspect(self, obj.clone())
                    .unwrap()
                    .as_ref()
                    .try_into()
                    .unwrap_or_else(|_| "(uninspectable)".into());
                eprintln!("  R{}: {}", i, inspect);
            } else {
                break;
            }
        }
        // eprintln!("Current CallInfo: {:?}", self.current_callinfo);
        eprintln!("Target Class: {}", self.target_class.name());
        // eprintln!("Exception: {:?}", self.exception);
        eprintln!("--- Breadcrumb ---");
        if let Some(bc) = &self.current_breadcrumb {
            bc.display_breadcrumb_for_debug(0, max_breadcrumb_level);
        } else {
            eprintln!("(none)");
        }
        eprintln!("=== End of VM Dump ===");
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
        catch_target_pos: Vec::new(),
    };
    for sym in irep.syms.iter() {
        irep1
            .syms
            .push(RSym::new(sym.to_string_lossy().to_string()));
    }
    for str in irep.strvals.iter() {
        irep1
            .pool
            .push(RPool::Str(str.to_string_lossy().to_string()));
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
