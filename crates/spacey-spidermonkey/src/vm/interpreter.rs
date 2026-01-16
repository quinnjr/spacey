//! The bytecode interpreter.

use crate::Error;
use crate::builtins::{BuiltinId, call_builtin, json_parse};
use crate::compiler::bytecode::CompiledFunction;
use crate::compiler::{Bytecode, OpCode, Operand};
use crate::runtime::object::Object;
use crate::runtime::value::Value;
use rustc_hash::FxHashMap;

/// A call frame representing a function invocation.
#[derive(Debug)]
struct CallFrame {
    /// Return address (instruction pointer after call)
    return_ip: usize,
    /// Stack base pointer for this frame
    stack_base: usize,
    /// Local variables for this frame
    locals: Vec<Value>,
    /// Index of the function being executed (for nested calls)
    #[allow(dead_code)]
    func_idx: Option<usize>,
    /// The bytecode being executed in this frame
    #[allow(dead_code)]
    bytecode_idx: usize,
    /// The 'this' value for this frame
    this_value: Value,
}

/// The virtual machine that executes bytecode.
pub struct VM {
    /// The value stack
    stack: Vec<Value>,
    /// Instruction pointer
    ip: usize,
    /// Global variables
    globals: FxHashMap<String, Value>,
    /// Call frames
    call_stack: Vec<CallFrame>,
    /// Object heap (simple allocation)
    objects: Vec<Object>,
    /// The 'this' value for the current context
    this_value: Value,
    /// Compiled functions from bytecode
    functions: Vec<CompiledFunction>,
    /// Last property access receiver (for method calls)
    last_receiver: Option<Value>,
}

impl VM {
    /// Creates a new VM with built-in objects initialized.
    pub fn new() -> Self {
        let mut vm = Self {
            stack: Vec::with_capacity(256),
            ip: 0,
            globals: FxHashMap::default(),
            call_stack: Vec::new(),
            objects: Vec::new(),
            this_value: Value::Undefined,
            functions: Vec::new(),
            last_receiver: None,
        };
        vm.init_builtins();
        vm
    }

    /// Initialize built-in objects and global functions.
    fn init_builtins(&mut self) {
        // Global functions
        self.globals.insert(
            "parseInt".to_string(),
            Value::NativeFunction(BuiltinId::ParseInt as u16),
        );
        self.globals.insert(
            "parseFloat".to_string(),
            Value::NativeFunction(BuiltinId::ParseFloat as u16),
        );
        self.globals.insert(
            "isNaN".to_string(),
            Value::NativeFunction(BuiltinId::IsNaN as u16),
        );
        self.globals.insert(
            "isFinite".to_string(),
            Value::NativeFunction(BuiltinId::IsFinite as u16),
        );
        self.globals.insert(
            "encodeURI".to_string(),
            Value::NativeFunction(BuiltinId::EncodeURI as u16),
        );
        self.globals.insert(
            "decodeURI".to_string(),
            Value::NativeFunction(BuiltinId::DecodeURI as u16),
        );
        self.globals.insert(
            "encodeURIComponent".to_string(),
            Value::NativeFunction(BuiltinId::EncodeURIComponent as u16),
        );
        self.globals.insert(
            "decodeURIComponent".to_string(),
            Value::NativeFunction(BuiltinId::DecodeURIComponent as u16),
        );

        // Create Math object
        let mut math_obj = Object::new();
        // Math constants
        math_obj.set("E".to_string(), Value::Number(std::f64::consts::E));
        math_obj.set("PI".to_string(), Value::Number(std::f64::consts::PI));
        math_obj.set("LN2".to_string(), Value::Number(std::f64::consts::LN_2));
        math_obj.set("LN10".to_string(), Value::Number(std::f64::consts::LN_10));
        math_obj.set("LOG2E".to_string(), Value::Number(std::f64::consts::LOG2_E));
        math_obj.set(
            "LOG10E".to_string(),
            Value::Number(std::f64::consts::LOG10_E),
        );
        math_obj.set("SQRT2".to_string(), Value::Number(std::f64::consts::SQRT_2));
        math_obj.set(
            "SQRT1_2".to_string(),
            Value::Number(std::f64::consts::FRAC_1_SQRT_2),
        );
        // Math methods
        math_obj.set(
            "abs".to_string(),
            Value::NativeFunction(BuiltinId::MathAbs as u16),
        );
        math_obj.set(
            "ceil".to_string(),
            Value::NativeFunction(BuiltinId::MathCeil as u16),
        );
        math_obj.set(
            "floor".to_string(),
            Value::NativeFunction(BuiltinId::MathFloor as u16),
        );
        math_obj.set(
            "round".to_string(),
            Value::NativeFunction(BuiltinId::MathRound as u16),
        );
        math_obj.set(
            "max".to_string(),
            Value::NativeFunction(BuiltinId::MathMax as u16),
        );
        math_obj.set(
            "min".to_string(),
            Value::NativeFunction(BuiltinId::MathMin as u16),
        );
        math_obj.set(
            "pow".to_string(),
            Value::NativeFunction(BuiltinId::MathPow as u16),
        );
        math_obj.set(
            "sqrt".to_string(),
            Value::NativeFunction(BuiltinId::MathSqrt as u16),
        );
        math_obj.set(
            "exp".to_string(),
            Value::NativeFunction(BuiltinId::MathExp as u16),
        );
        math_obj.set(
            "log".to_string(),
            Value::NativeFunction(BuiltinId::MathLog as u16),
        );
        math_obj.set(
            "sin".to_string(),
            Value::NativeFunction(BuiltinId::MathSin as u16),
        );
        math_obj.set(
            "cos".to_string(),
            Value::NativeFunction(BuiltinId::MathCos as u16),
        );
        math_obj.set(
            "tan".to_string(),
            Value::NativeFunction(BuiltinId::MathTan as u16),
        );
        math_obj.set(
            "asin".to_string(),
            Value::NativeFunction(BuiltinId::MathAsin as u16),
        );
        math_obj.set(
            "acos".to_string(),
            Value::NativeFunction(BuiltinId::MathAcos as u16),
        );
        math_obj.set(
            "atan".to_string(),
            Value::NativeFunction(BuiltinId::MathAtan as u16),
        );
        math_obj.set(
            "atan2".to_string(),
            Value::NativeFunction(BuiltinId::MathAtan2 as u16),
        );
        math_obj.set(
            "random".to_string(),
            Value::NativeFunction(BuiltinId::MathRandom as u16),
        );
        math_obj.set(
            "sign".to_string(),
            Value::NativeFunction(BuiltinId::MathSign as u16),
        );
        math_obj.set(
            "trunc".to_string(),
            Value::NativeFunction(BuiltinId::MathTrunc as u16),
        );

        let math_idx = self.objects.len();
        self.objects.push(math_obj);
        self.globals
            .insert("Math".to_string(), Value::Object(math_idx));

        // Number constants
        self.globals
            .insert("NaN".to_string(), Value::Number(f64::NAN));
        self.globals
            .insert("Infinity".to_string(), Value::Number(f64::INFINITY));
        self.globals
            .insert("undefined".to_string(), Value::Undefined);

        // Create Array constructor object
        let mut array_obj = Object::new();
        array_obj.set(
            "isArray".to_string(),
            Value::NativeFunction(BuiltinId::ArrayIsArray as u16),
        );
        let array_idx = self.objects.len();
        self.objects.push(array_obj);
        self.globals
            .insert("Array".to_string(), Value::Object(array_idx));

        // Create Object constructor
        let mut object_obj = Object::new();
        object_obj.set(
            "keys".to_string(),
            Value::NativeFunction(BuiltinId::ObjectKeys as u16),
        );
        object_obj.set(
            "values".to_string(),
            Value::NativeFunction(BuiltinId::ObjectValues as u16),
        );
        object_obj.set(
            "entries".to_string(),
            Value::NativeFunction(BuiltinId::ObjectEntries as u16),
        );
        object_obj.set(
            "create".to_string(),
            Value::NativeFunction(BuiltinId::ObjectCreate as u16),
        );
        object_obj.set(
            "defineProperty".to_string(),
            Value::NativeFunction(BuiltinId::ObjectDefineProperty as u16),
        );
        object_obj.set(
            "getOwnPropertyDescriptor".to_string(),
            Value::NativeFunction(BuiltinId::ObjectGetOwnPropertyDescriptor as u16),
        );
        object_obj.set(
            "getPrototypeOf".to_string(),
            Value::NativeFunction(BuiltinId::ObjectGetPrototypeOf as u16),
        );
        let object_idx = self.objects.len();
        self.objects.push(object_obj);
        self.globals
            .insert("Object".to_string(), Value::Object(object_idx));

        // Create JSON object
        let mut json_obj = Object::new();
        json_obj.set(
            "parse".to_string(),
            Value::NativeFunction(BuiltinId::JsonParse as u16),
        );
        json_obj.set(
            "stringify".to_string(),
            Value::NativeFunction(BuiltinId::JsonStringify as u16),
        );
        let json_idx = self.objects.len();
        self.objects.push(json_obj);
        self.globals
            .insert("JSON".to_string(), Value::Object(json_idx));
    }

    /// Executes bytecode and returns the result.
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value, Error> {
        self.ip = 0;
        self.stack.clear();

        // Store functions from the bytecode
        self.functions = bytecode.functions.clone();

        // Execute with the main bytecode
        self.execute_bytecode(bytecode)
    }

    /// Internal method to execute a bytecode chunk.
    fn execute_bytecode(&mut self, bytecode: &Bytecode) -> Result<Value, Error> {
        loop {
            if self.ip >= bytecode.instructions.len() {
                break;
            }

            let instruction = &bytecode.instructions[self.ip];
            self.ip += 1;

            match instruction.opcode {
                OpCode::Halt => break,
                OpCode::Nop => {}

                // Stack operations
                OpCode::LoadConst => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = bytecode.constants[*idx as usize].clone();
                        self.stack.push(value);
                    }
                }

                OpCode::LoadUndefined => self.stack.push(Value::Undefined),
                OpCode::LoadNull => self.stack.push(Value::Null),
                OpCode::LoadTrue => self.stack.push(Value::Boolean(true)),
                OpCode::LoadFalse => self.stack.push(Value::Boolean(false)),

                OpCode::Pop => {
                    self.stack.pop();
                }

                OpCode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                // Arithmetic operations
                OpCode::Add => self.op_add()?,
                OpCode::Sub => self.binary_num_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_num_op(|a, b| a * b)?,
                OpCode::Div => self.binary_num_op(|a, b| a / b)?,
                OpCode::Mod => self.binary_num_op(|a, b| a % b)?,
                OpCode::Pow => self.binary_num_op(|a, b| a.powf(b))?,

                OpCode::Neg => {
                    let val = self.pop()?;
                    let num = self.to_number(&val);
                    self.stack.push(Value::Number(-num));
                }

                // Comparison operations
                OpCode::Lt => self.compare_op(|a, b| a < b)?,
                OpCode::Le => self.compare_op(|a, b| a <= b)?,
                OpCode::Gt => self.compare_op(|a, b| a > b)?,
                OpCode::Ge => self.compare_op(|a, b| a >= b)?,

                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.abstract_equality(&a, &b);
                    self.stack.push(Value::Boolean(result));
                }

                OpCode::Ne => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = !self.abstract_equality(&a, &b);
                    self.stack.push(Value::Boolean(result));
                }

                OpCode::StrictEq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.strict_equality(&a, &b);
                    self.stack.push(Value::Boolean(result));
                }

                OpCode::StrictNe => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = !self.strict_equality(&a, &b);
                    self.stack.push(Value::Boolean(result));
                }

                // Logical operations
                OpCode::Not => {
                    let val = self.pop()?;
                    self.stack.push(Value::Boolean(!val.to_boolean()));
                }

                // Bitwise operations
                OpCode::BitAnd => self.bitwise_op(|a, b| a & b)?,
                OpCode::BitOr => self.bitwise_op(|a, b| a | b)?,
                OpCode::BitXor => self.bitwise_op(|a, b| a ^ b)?,
                OpCode::BitNot => {
                    let val = self.pop()?;
                    let num = self.to_int32(&val);
                    self.stack.push(Value::Number((!num) as f64));
                }
                OpCode::Shl => self.shift_op(|a, b| a << (b & 0x1f))?,
                OpCode::Shr => self.shift_op(|a, b| a >> (b & 0x1f))?,
                OpCode::Ushr => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let a_u32 = self.to_uint32(&a);
                    let b_u32 = self.to_uint32(&b);
                    let result = a_u32 >> (b_u32 & 0x1f);
                    self.stack.push(Value::Number(result as f64));
                }

                // Variable operations
                OpCode::LoadLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        if let Some(frame) = self.call_stack.last() {
                            let value = frame
                                .locals
                                .get(*idx as usize)
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            self.stack.push(value);
                        } else {
                            // In global scope, locals are on the stack
                            let value = self
                                .stack
                                .get(*idx as usize)
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            self.stack.push(value);
                        }
                    }
                }

                OpCode::StoreLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let value = self.pop()?;
                        if let Some(frame) = self.call_stack.last_mut() {
                            let idx = *idx as usize;
                            if idx >= frame.locals.len() {
                                frame.locals.resize(idx + 1, Value::Undefined);
                            }
                            frame.locals[idx] = value;
                        } else {
                            // In global scope, locals are on the stack
                            let idx = *idx as usize;
                            if idx < self.stack.len() {
                                self.stack[idx] = value;
                            }
                        }
                    }
                }

                OpCode::LoadGlobal => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand
                        && let Value::String(name) = &bytecode.constants[*idx as usize]
                    {
                        let value = self.globals.get(name).cloned().unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }

                OpCode::StoreGlobal => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = self.pop()?;
                        if let Value::String(name) = &bytecode.constants[*idx as usize] {
                            self.globals.insert(name.clone(), value);
                        }
                    }
                }

                OpCode::LoadUpvalue | OpCode::StoreUpvalue => {
                    // Closures not fully implemented yet
                    self.stack.push(Value::Undefined);
                }

                // Property operations
                OpCode::GetProperty => {
                    let key = self.pop()?;
                    let obj = self.pop()?;
                    let value = self.get_property(&obj, &key);
                    // Store receiver for potential method call
                    self.last_receiver = Some(obj);
                    self.stack.push(value);
                }

                OpCode::SetProperty => {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    let obj = self.pop()?;
                    self.set_property(obj, &key, value.clone());
                    self.stack.push(value);
                }

                OpCode::DeleteProperty => {
                    let _key = self.pop()?;
                    let _obj = self.pop()?;
                    // Simplified: always return true
                    self.stack.push(Value::Boolean(true));
                }

                // Control flow
                OpCode::Jump => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand {
                        self.ip = (self.ip as i32 + offset) as usize;
                    }
                }

                OpCode::JumpIfFalse => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand {
                        let value = self
                            .stack
                            .last()
                            .ok_or(Error::InternalError("Stack underflow".into()))?;
                        if !value.to_boolean() {
                            self.ip = (self.ip as i32 + offset) as usize;
                        }
                    }
                }

                OpCode::JumpIfTrue => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand {
                        let value = self
                            .stack
                            .last()
                            .ok_or(Error::InternalError("Stack underflow".into()))?;
                        if value.to_boolean() {
                            self.ip = (self.ip as i32 + offset) as usize;
                        }
                    }
                }

                OpCode::JumpIfNotUndefined => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand {
                        let value = self
                            .stack
                            .last()
                            .ok_or(Error::InternalError("Stack underflow".into()))?;
                        if !matches!(value, Value::Undefined) {
                            self.ip = (self.ip as i32 + offset) as usize;
                        }
                    }
                }

                // Function operations
                OpCode::Call => {
                    if let Some(Operand::ArgCount(argc)) = &instruction.operand {
                        let argc = *argc as usize;

                        // Pop arguments in reverse order
                        let mut args = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            args.push(self.pop().unwrap_or(Value::Undefined));
                        }
                        args.reverse();

                        // Pop the function
                        let callee = self.pop()?;

                        match callee {
                            Value::Function(func_idx) => {
                                // Get the function
                                if func_idx >= self.functions.len() {
                                    return Err(Error::TypeError("Invalid function index".into()));
                                }

                                // Clone the function to avoid borrow issues
                                let func = self.functions[func_idx].clone();

                                // Set up locals with arguments
                                let mut locals =
                                    vec![Value::Undefined; func.local_count.max(func.params.len())];
                                for (i, arg) in args.into_iter().enumerate() {
                                    if i < locals.len() {
                                        locals[i] = arg;
                                    }
                                }

                                // Save current state and create call frame
                                let frame = CallFrame {
                                    return_ip: self.ip,
                                    stack_base: self.stack.len(),
                                    locals,
                                    func_idx: Some(func_idx),
                                    bytecode_idx: 0, // Main bytecode
                                    this_value: if func.is_arrow {
                                        self.this_value.clone()
                                    } else {
                                        Value::Undefined
                                    },
                                };
                                self.call_stack.push(frame);

                                // Save the current IP and execute the function
                                let saved_ip = self.ip;
                                let result = self.execute_function(&func)?;
                                self.ip = saved_ip; // Restore IP to continue main bytecode

                                // Pop the call frame we pushed
                                self.call_stack.pop();

                                // Push result
                                self.stack.push(result);
                            }
                            Value::NativeFunction(builtin_id) => {
                                // Call native function with receiver as this
                                if let Some(id) = BuiltinId::from_u16(builtin_id) {
                                    let this = self
                                        .last_receiver
                                        .take()
                                        .unwrap_or(self.this_value.clone());
                                    // Handle array methods specially (they need object heap access)
                                    let result = if self.is_array_method(id) {
                                        self.call_array_method(id, &this, &args)?
                                    } else if self.is_object_static_method(id) {
                                        self.call_object_static_method(id, &args)?
                                    } else if self.is_json_method(id) {
                                        self.call_json_method(id, &args)?
                                    } else {
                                        call_builtin(id, &this, &args)?
                                    };
                                    self.stack.push(result);
                                } else {
                                    return Err(Error::TypeError("Invalid native function".into()));
                                }
                            }
                            _ => {
                                return Err(Error::TypeError(format!(
                                    "{} is not a function",
                                    callee.type_of()
                                )));
                            }
                        }
                    }
                }

                OpCode::Return => {
                    let return_value = self.pop().unwrap_or(Value::Undefined);

                    if let Some(frame) = self.call_stack.pop() {
                        // Restore state
                        self.ip = frame.return_ip;
                        self.stack.truncate(frame.stack_base);
                        self.this_value = frame.this_value;
                        return Ok(return_value);
                    } else {
                        // Return from top-level
                        return Ok(return_value);
                    }
                }

                OpCode::Closure => {
                    if let Some(Operand::Function(func_idx)) = &instruction.operand {
                        self.stack.push(Value::Function(*func_idx as usize));
                    } else {
                        self.stack.push(Value::Undefined);
                    }
                }

                // Object operations
                OpCode::NewObject => {
                    let obj = Object::new();
                    let idx = self.objects.len();
                    self.objects.push(obj);
                    self.stack.push(Value::Object(idx));
                }

                OpCode::NewArray => {
                    if let Some(Operand::ArgCount(count)) = &instruction.operand {
                        let count = *count as usize;
                        let mut elements = Vec::with_capacity(count);

                        // Pop elements in reverse order
                        for _ in 0..count {
                            elements.push(self.pop().unwrap_or(Value::Undefined));
                        }
                        elements.reverse();

                        // Create array object
                        let mut obj = Object::new();
                        for (i, elem) in elements.into_iter().enumerate() {
                            obj.set(i.to_string(), elem);
                        }
                        obj.set("length".to_string(), Value::Number(count as f64));

                        let idx = self.objects.len();
                        self.objects.push(obj);
                        self.stack.push(Value::Object(idx));
                    }
                }

                OpCode::TypeOf => {
                    let val = self.pop()?;
                    let type_str = val.type_of();
                    self.stack.push(Value::String(type_str.to_string()));
                }

                OpCode::InstanceOf => {
                    let _constructor = self.pop()?;
                    let _obj = self.pop()?;
                    // Simplified: always false for now
                    self.stack.push(Value::Boolean(false));
                }

                OpCode::In => {
                    let obj = self.pop()?;
                    let key = self.pop()?;
                    let result = self.has_property(&obj, &key);
                    self.stack.push(Value::Boolean(result));
                }

                // Special
                OpCode::LoadThis => {
                    self.stack.push(self.this_value.clone());
                }

                OpCode::Throw => {
                    let value = self.pop()?;
                    return Err(Error::InternalError(format!(
                        "Uncaught exception: {:?}",
                        value
                    )));
                }
            }
        }

        self.stack
            .pop()
            .ok_or(Error::InternalError("No result".into()))
    }

    /// Execute a function's bytecode.
    fn execute_function(&mut self, func: &CompiledFunction) -> Result<Value, Error> {
        let bytecode = &func.bytecode;

        // Register nested functions and track their base index
        let nested_func_base = self.functions.len();
        for nested_func in &bytecode.functions {
            self.functions.push(nested_func.clone());
        }

        self.ip = 0;

        let result = self.execute_function_inner(bytecode, nested_func_base);

        // Clean up nested functions (restore to original state)
        self.functions.truncate(nested_func_base);

        result
    }

    /// Inner function execution loop.
    fn execute_function_inner(
        &mut self,
        bytecode: &Bytecode,
        nested_func_base: usize,
    ) -> Result<Value, Error> {
        loop {
            if self.ip >= bytecode.instructions.len() {
                break;
            }

            let instruction = &bytecode.instructions[self.ip];
            self.ip += 1;

            match instruction.opcode {
                OpCode::Halt => break,
                OpCode::Nop => {}
                OpCode::Return => {
                    return self.pop().or(Ok(Value::Undefined));
                }

                // Stack operations
                OpCode::LoadConst => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = bytecode.constants[*idx as usize].clone();
                        self.stack.push(value);
                    }
                }

                OpCode::LoadUndefined => self.stack.push(Value::Undefined),
                OpCode::LoadNull => self.stack.push(Value::Null),
                OpCode::LoadTrue => self.stack.push(Value::Boolean(true)),
                OpCode::LoadFalse => self.stack.push(Value::Boolean(false)),
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                // Arithmetic
                OpCode::Add => self.op_add()?,
                OpCode::Sub => self.binary_num_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_num_op(|a, b| a * b)?,
                OpCode::Div => self.binary_num_op(|a, b| a / b)?,
                OpCode::Mod => self.binary_num_op(|a, b| a % b)?,
                OpCode::Pow => self.binary_num_op(|a, b| a.powf(b))?,
                OpCode::Neg => {
                    let val = self.pop()?;
                    let num = self.to_number(&val);
                    self.stack.push(Value::Number(-num));
                }

                // Comparisons
                OpCode::Lt => self.compare_op(|a, b| a < b)?,
                OpCode::Le => self.compare_op(|a, b| a <= b)?,
                OpCode::Gt => self.compare_op(|a, b| a > b)?,
                OpCode::Ge => self.compare_op(|a, b| a >= b)?,
                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(Value::Boolean(self.abstract_equality(&a, &b)));
                }
                OpCode::Ne => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(Value::Boolean(!self.abstract_equality(&a, &b)));
                }
                OpCode::StrictEq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(Value::Boolean(self.strict_equality(&a, &b)));
                }
                OpCode::StrictNe => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack
                        .push(Value::Boolean(!self.strict_equality(&a, &b)));
                }

                // Logical & Bitwise
                OpCode::Not => {
                    let val = self.pop()?;
                    self.stack.push(Value::Boolean(!val.to_boolean()));
                }
                OpCode::BitAnd => self.bitwise_op(|a, b| a & b)?,
                OpCode::BitOr => self.bitwise_op(|a, b| a | b)?,
                OpCode::BitXor => self.bitwise_op(|a, b| a ^ b)?,
                OpCode::BitNot => {
                    let val = self.pop()?;
                    let num = self.to_int32(&val);
                    self.stack.push(Value::Number((!num) as f64));
                }
                OpCode::Shl => self.shift_op(|a, b| a << (b & 0x1f))?,
                OpCode::Shr => self.shift_op(|a, b| a >> (b & 0x1f))?,
                OpCode::Ushr => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let a_u32 = self.to_uint32(&a);
                    let b_u32 = self.to_uint32(&b);
                    self.stack
                        .push(Value::Number((a_u32 >> (b_u32 & 0x1f)) as f64));
                }

                // Local variables (within function scope)
                OpCode::LoadLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        if let Some(frame) = self.call_stack.last() {
                            let value = frame
                                .locals
                                .get(*idx as usize)
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            self.stack.push(value);
                        } else {
                            self.stack.push(Value::Undefined);
                        }
                    }
                }
                OpCode::StoreLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let value = self.pop()?;
                        if let Some(frame) = self.call_stack.last_mut() {
                            let idx = *idx as usize;
                            if idx >= frame.locals.len() {
                                frame.locals.resize(idx + 1, Value::Undefined);
                            }
                            frame.locals[idx] = value;
                        }
                    }
                }

                // Global variables
                OpCode::LoadGlobal => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand
                        && let Value::String(name) = &bytecode.constants[*idx as usize]
                    {
                        let value = self.globals.get(name).cloned().unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }
                OpCode::StoreGlobal => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = self.pop()?;
                        if let Value::String(name) = &bytecode.constants[*idx as usize] {
                            self.globals.insert(name.clone(), value);
                        }
                    }
                }

                OpCode::LoadUpvalue | OpCode::StoreUpvalue => {
                    // Closures not fully implemented yet
                    self.stack.push(Value::Undefined);
                }

                // Property operations
                OpCode::GetProperty => {
                    let key = self.pop()?;
                    let obj = self.pop()?;
                    let value = self.get_property(&obj, &key);
                    // Store receiver for potential method call
                    self.last_receiver = Some(obj);
                    self.stack.push(value);
                }
                OpCode::SetProperty => {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    let obj = self.pop()?;
                    self.set_property(obj, &key, value.clone());
                    self.stack.push(value);
                }
                OpCode::DeleteProperty => {
                    let _key = self.pop()?;
                    let _obj = self.pop()?;
                    self.stack.push(Value::Boolean(true));
                }

                // Control flow
                OpCode::Jump => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand {
                        self.ip = (self.ip as i32 + offset) as usize;
                    }
                }
                OpCode::JumpIfFalse => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand
                        && let Some(value) = self.stack.last()
                        && !value.to_boolean()
                    {
                        self.ip = (self.ip as i32 + offset) as usize;
                    }
                }
                OpCode::JumpIfTrue => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand
                        && let Some(value) = self.stack.last()
                        && value.to_boolean()
                    {
                        self.ip = (self.ip as i32 + offset) as usize;
                    }
                }
                OpCode::JumpIfNotUndefined => {
                    if let Some(Operand::Jump(offset)) = &instruction.operand
                        && let Some(value) = self.stack.last()
                        && !matches!(value, Value::Undefined)
                    {
                        self.ip = (self.ip as i32 + offset) as usize;
                    }
                }

                // Nested function calls
                OpCode::Call => {
                    if let Some(Operand::ArgCount(argc)) = &instruction.operand {
                        let argc = *argc as usize;
                        let mut args = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            args.push(self.pop().unwrap_or(Value::Undefined));
                        }
                        args.reverse();
                        let callee = self.pop()?;

                        match callee {
                            Value::Function(func_idx) => {
                                if func_idx >= self.functions.len() {
                                    return Err(Error::TypeError("Invalid function index".into()));
                                }
                                let nested_func = self.functions[func_idx].clone();
                                let mut locals =
                                    vec![
                                        Value::Undefined;
                                        nested_func.local_count.max(nested_func.params.len())
                                    ];
                                for (i, arg) in args.into_iter().enumerate() {
                                    if i < locals.len() {
                                        locals[i] = arg;
                                    }
                                }
                                let frame = CallFrame {
                                    return_ip: self.ip,
                                    stack_base: self.stack.len(),
                                    locals,
                                    func_idx: Some(func_idx),
                                    bytecode_idx: 0,
                                    this_value: if nested_func.is_arrow {
                                        self.this_value.clone()
                                    } else {
                                        Value::Undefined
                                    },
                                };
                                self.call_stack.push(frame);
                                let saved_ip = self.ip;
                                let result = self.execute_function(&nested_func)?;
                                self.ip = saved_ip;
                                self.call_stack.pop();
                                self.stack.push(result);
                            }
                            Value::NativeFunction(builtin_id) => {
                                // Call native function with receiver as this
                                if let Some(id) = BuiltinId::from_u16(builtin_id) {
                                    let this = self
                                        .last_receiver
                                        .take()
                                        .unwrap_or(self.this_value.clone());
                                    // Handle array methods specially (they need object heap access)
                                    let result = if self.is_array_method(id) {
                                        self.call_array_method(id, &this, &args)?
                                    } else if self.is_object_static_method(id) {
                                        self.call_object_static_method(id, &args)?
                                    } else if self.is_json_method(id) {
                                        self.call_json_method(id, &args)?
                                    } else {
                                        call_builtin(id, &this, &args)?
                                    };
                                    self.stack.push(result);
                                } else {
                                    return Err(Error::TypeError("Invalid native function".into()));
                                }
                            }
                            _ => {
                                return Err(Error::TypeError(format!(
                                    "{} is not a function",
                                    callee.type_of()
                                )));
                            }
                        }
                    }
                }

                OpCode::Closure => {
                    if let Some(Operand::Function(func_idx)) = &instruction.operand {
                        // Nested functions use offset from nested_func_base
                        let actual_idx = nested_func_base + *func_idx as usize;
                        self.stack.push(Value::Function(actual_idx));
                    } else {
                        self.stack.push(Value::Undefined);
                    }
                }

                // Object operations
                OpCode::NewObject => {
                    let obj = Object::new();
                    let idx = self.objects.len();
                    self.objects.push(obj);
                    self.stack.push(Value::Object(idx));
                }
                OpCode::NewArray => {
                    if let Some(Operand::ArgCount(count)) = &instruction.operand {
                        let count = *count as usize;
                        let mut elements = Vec::with_capacity(count);
                        for _ in 0..count {
                            elements.push(self.pop().unwrap_or(Value::Undefined));
                        }
                        elements.reverse();
                        let mut obj = Object::new();
                        for (i, elem) in elements.into_iter().enumerate() {
                            obj.set(i.to_string(), elem);
                        }
                        obj.set("length".to_string(), Value::Number(count as f64));
                        let idx = self.objects.len();
                        self.objects.push(obj);
                        self.stack.push(Value::Object(idx));
                    }
                }
                OpCode::TypeOf => {
                    let val = self.pop()?;
                    self.stack.push(Value::String(val.type_of().to_string()));
                }
                OpCode::InstanceOf => {
                    let _constructor = self.pop()?;
                    let _obj = self.pop()?;
                    self.stack.push(Value::Boolean(false));
                }
                OpCode::In => {
                    let obj = self.pop()?;
                    let key = self.pop()?;
                    self.stack
                        .push(Value::Boolean(self.has_property(&obj, &key)));
                }

                OpCode::LoadThis => {
                    if let Some(frame) = self.call_stack.last() {
                        self.stack.push(frame.this_value.clone());
                    } else {
                        self.stack.push(self.this_value.clone());
                    }
                }

                OpCode::Throw => {
                    let value = self.pop()?;
                    return Err(Error::InternalError(format!(
                        "Uncaught exception: {:?}",
                        value
                    )));
                }
            }
        }

        // If we reach here without a return, return undefined
        Ok(self.stack.pop().unwrap_or(Value::Undefined))
    }

    // ==================== Helper Methods ====================

    fn pop(&mut self) -> Result<Value, Error> {
        self.stack
            .pop()
            .ok_or(Error::InternalError("Stack underflow".into()))
    }

    fn op_add(&mut self) -> Result<(), Error> {
        let b = self.pop()?;
        let a = self.pop()?;

        // String concatenation
        match (&a, &b) {
            (Value::String(s1), Value::String(s2)) => {
                self.stack.push(Value::String(format!("{}{}", s1, s2)));
            }
            (Value::String(s), other) | (other, Value::String(s)) => {
                let other_str = self.to_string(other);
                if matches!(&a, Value::String(_)) {
                    self.stack
                        .push(Value::String(format!("{}{}", s, other_str)));
                } else {
                    self.stack
                        .push(Value::String(format!("{}{}", other_str, s)));
                }
            }
            _ => {
                let a_num = self.to_number(&a);
                let b_num = self.to_number(&b);
                self.stack.push(Value::Number(a_num + b_num));
            }
        }

        Ok(())
    }

    fn binary_num_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        self.stack.push(Value::Number(op(a_num, b_num)));

        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(f64, f64) -> bool,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        self.stack.push(Value::Boolean(op(a_num, b_num)));

        Ok(())
    }

    fn bitwise_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(i32, i32) -> i32,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        let a_int = self.to_int32(&a);
        let b_int = self.to_int32(&b);
        self.stack.push(Value::Number(op(a_int, b_int) as f64));

        Ok(())
    }

    fn shift_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(i32, u32) -> i32,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        let a_int = self.to_int32(&a);
        let b_uint = self.to_uint32(&b);
        self.stack.push(Value::Number(op(a_int, b_uint) as f64));

        Ok(())
    }

    // ==================== Type Coercion ====================

    fn to_number(&self, value: &Value) -> f64 {
        match value {
            Value::Number(n) => *n,
            Value::Boolean(true) => 1.0,
            Value::Boolean(false) => 0.0,
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(s) => s.parse().unwrap_or(f64::NAN),
            Value::Symbol(_)
            | Value::BigInt(_)
            | Value::Object(_)
            | Value::Function(_)
            | Value::NativeFunction(_)
            | Value::Array(_)
            | Value::ParsedObject(_) => f64::NAN,
        }
    }

    fn to_int32(&self, value: &Value) -> i32 {
        let num = self.to_number(value);
        if num.is_nan() || num.is_infinite() || num == 0.0 {
            return 0;
        }
        let int = num.trunc() as i64;
        int as i32
    }

    fn to_uint32(&self, value: &Value) -> u32 {
        let num = self.to_number(value);
        if num.is_nan() || num.is_infinite() || num == 0.0 {
            return 0;
        }
        let int = num.trunc() as i64;
        int as u32
    }

    fn to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if *n > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
                } else {
                    n.to_string()
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Undefined => "undefined".to_string(),
            Value::Symbol(id) => format!("Symbol({})", id),
            Value::BigInt(s) => s.clone(),
            Value::Object(_) | Value::ParsedObject(_) => "[object Object]".to_string(),
            Value::Function(_) | Value::NativeFunction(_) => "[Function]".to_string(),
            Value::Array(arr) => arr
                .iter()
                .map(|v| self.to_string(v))
                .collect::<Vec<_>>()
                .join(","),
        }
    }

    // ==================== Equality ====================

    fn strict_equality(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Number(x), Value::Number(y)) => {
                if x.is_nan() || y.is_nan() {
                    false
                } else {
                    x == y
                }
            }
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::Object(x), Value::Object(y)) => x == y,
            (Value::Symbol(x), Value::Symbol(y)) => x == y,
            _ => false,
        }
    }

    fn abstract_equality(&self, a: &Value, b: &Value) -> bool {
        // Same type: use strict equality
        if std::mem::discriminant(a) == std::mem::discriminant(b) {
            return self.strict_equality(a, b);
        }

        // null == undefined
        match (a, b) {
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => return true,
            _ => {}
        }

        // Number comparisons with type coercion
        match (a, b) {
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                let s_num: f64 = s.parse().unwrap_or(f64::NAN);
                if s_num.is_nan() || n.is_nan() {
                    false
                } else {
                    n == &s_num
                }
            }
            (Value::Boolean(b_val), other) | (other, Value::Boolean(b_val)) => {
                let b_num = if *b_val { 1.0 } else { 0.0 };
                self.abstract_equality(&Value::Number(b_num), other)
            }
            _ => false,
        }
    }

    // ==================== Property Access ====================

    fn get_property(&self, obj: &Value, key: &Value) -> Value {
        let key_str = self.to_string(key);

        match obj {
            Value::Object(idx) => {
                if let Some(object) = self.objects.get(*idx) {
                    // Check for array methods first
                    if object.has("length")
                        && let Some(method) = self.get_array_method(&key_str)
                    {
                        return method;
                    }
                    object.get(&key_str).cloned().unwrap_or(Value::Undefined)
                } else {
                    Value::Undefined
                }
            }
            Value::String(s) => {
                // String properties and methods
                if key_str == "length" {
                    Value::Number(s.chars().count() as f64)
                } else if let Ok(idx) = key_str.parse::<usize>() {
                    s.chars()
                        .nth(idx)
                        .map(|c| Value::String(c.to_string()))
                        .unwrap_or(Value::Undefined)
                } else if let Some(method) = self.get_string_method(&key_str) {
                    method
                } else {
                    Value::Undefined
                }
            }
            Value::Number(_) => {
                // Number prototype methods
                if let Some(method) = self.get_number_method(&key_str) {
                    method
                } else {
                    Value::Undefined
                }
            }
            _ => Value::Undefined,
        }
    }

    /// Get a String prototype method by name.
    fn get_string_method(&self, name: &str) -> Option<Value> {
        let id = match name {
            "charAt" => BuiltinId::StringCharAt,
            "charCodeAt" => BuiltinId::StringCharCodeAt,
            "concat" => BuiltinId::StringConcat,
            "indexOf" => BuiltinId::StringIndexOf,
            "lastIndexOf" => BuiltinId::StringLastIndexOf,
            "slice" => BuiltinId::StringSlice,
            "substring" => BuiltinId::StringSubstring,
            "substr" => BuiltinId::StringSubstr,
            "toLowerCase" => BuiltinId::StringToLowerCase,
            "toUpperCase" => BuiltinId::StringToUpperCase,
            "trim" => BuiltinId::StringTrim,
            "split" => BuiltinId::StringSplit,
            "replace" => BuiltinId::StringReplace,
            "match" => BuiltinId::StringMatch,
            "search" => BuiltinId::StringSearch,
            "repeat" => BuiltinId::StringRepeat,
            "startsWith" => BuiltinId::StringStartsWith,
            "endsWith" => BuiltinId::StringEndsWith,
            "includes" => BuiltinId::StringIncludes,
            "padStart" => BuiltinId::StringPadStart,
            "padEnd" => BuiltinId::StringPadEnd,
            _ => return None,
        };
        Some(Value::NativeFunction(id as u16))
    }

    /// Get a Number prototype method by name.
    fn get_number_method(&self, name: &str) -> Option<Value> {
        let id = match name {
            "toString" => BuiltinId::NumberToString,
            "toFixed" => BuiltinId::NumberToFixed,
            "toExponential" => BuiltinId::NumberToExponential,
            "toPrecision" => BuiltinId::NumberToPrecision,
            "valueOf" => BuiltinId::NumberValueOf,
            _ => return None,
        };
        Some(Value::NativeFunction(id as u16))
    }

    /// Get an Array prototype method by name.
    fn get_array_method(&self, name: &str) -> Option<Value> {
        let id = match name {
            "push" => BuiltinId::ArrayPush,
            "pop" => BuiltinId::ArrayPop,
            "shift" => BuiltinId::ArrayShift,
            "unshift" => BuiltinId::ArrayUnshift,
            "slice" => BuiltinId::ArraySlice,
            "splice" => BuiltinId::ArraySplice,
            "concat" => BuiltinId::ArrayConcat,
            "join" => BuiltinId::ArrayJoin,
            "reverse" => BuiltinId::ArrayReverse,
            "sort" => BuiltinId::ArraySort,
            "indexOf" => BuiltinId::ArrayIndexOf,
            "lastIndexOf" => BuiltinId::ArrayLastIndexOf,
            "forEach" => BuiltinId::ArrayForEach,
            "map" => BuiltinId::ArrayMap,
            "filter" => BuiltinId::ArrayFilter,
            "every" => BuiltinId::ArrayEvery,
            "some" => BuiltinId::ArraySome,
            "reduce" => BuiltinId::ArrayReduce,
            "reduceRight" => BuiltinId::ArrayReduceRight,
            _ => return None,
        };
        Some(Value::NativeFunction(id as u16))
    }

    /// Check if a builtin ID is an array method.
    fn is_array_method(&self, id: BuiltinId) -> bool {
        matches!(
            id,
            BuiltinId::ArrayPush
                | BuiltinId::ArrayPop
                | BuiltinId::ArrayShift
                | BuiltinId::ArrayUnshift
                | BuiltinId::ArraySlice
                | BuiltinId::ArraySplice
                | BuiltinId::ArrayConcat
                | BuiltinId::ArrayJoin
                | BuiltinId::ArrayReverse
                | BuiltinId::ArraySort
                | BuiltinId::ArrayIndexOf
                | BuiltinId::ArrayLastIndexOf
                | BuiltinId::ArrayForEach
                | BuiltinId::ArrayMap
                | BuiltinId::ArrayFilter
                | BuiltinId::ArrayEvery
                | BuiltinId::ArraySome
                | BuiltinId::ArrayReduce
                | BuiltinId::ArrayReduceRight
        )
    }

    /// Call an array method with access to the object heap.
    fn call_array_method(
        &mut self,
        id: BuiltinId,
        this: &Value,
        args: &[Value],
    ) -> Result<Value, Error> {
        let idx = match this {
            Value::Object(idx) => *idx,
            _ => return Err(Error::TypeError("Array method called on non-object".into())),
        };

        match id {
            BuiltinId::ArrayPush => {
                let obj = self
                    .objects
                    .get_mut(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                for (i, arg) in args.iter().enumerate() {
                    obj.set((len + i).to_string(), arg.clone());
                }
                let new_len = len + args.len();
                obj.set("length".to_string(), Value::Number(new_len as f64));
                Ok(Value::Number(new_len as f64))
            }
            BuiltinId::ArrayPop => {
                let obj = self
                    .objects
                    .get_mut(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                if len == 0 {
                    return Ok(Value::Undefined);
                }

                let last_idx = (len - 1).to_string();
                let value = obj.get(&last_idx).cloned().unwrap_or(Value::Undefined);
                obj.delete(&last_idx);
                obj.set("length".to_string(), Value::Number((len - 1) as f64));
                Ok(value)
            }
            BuiltinId::ArrayShift => {
                let obj = self
                    .objects
                    .get_mut(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                if len == 0 {
                    return Ok(Value::Undefined);
                }

                let first = obj.get("0").cloned().unwrap_or(Value::Undefined);
                // Shift all elements down
                for i in 1..len {
                    if let Some(v) = obj.get(&i.to_string()).cloned() {
                        obj.set((i - 1).to_string(), v);
                    } else {
                        obj.delete(&(i - 1).to_string());
                    }
                }
                obj.delete(&(len - 1).to_string());
                obj.set("length".to_string(), Value::Number((len - 1) as f64));
                Ok(first)
            }
            BuiltinId::ArrayUnshift => {
                let obj = self
                    .objects
                    .get_mut(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let shift = args.len();
                // Shift existing elements up
                for i in (0..len).rev() {
                    if let Some(v) = obj.get(&i.to_string()).cloned() {
                        obj.set((i + shift).to_string(), v);
                    }
                }
                // Insert new elements at the beginning
                for (i, arg) in args.iter().enumerate() {
                    obj.set(i.to_string(), arg.clone());
                }
                let new_len = len + shift;
                obj.set("length".to_string(), Value::Number(new_len as f64));
                Ok(Value::Number(new_len as f64))
            }
            BuiltinId::ArrayJoin => {
                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let separator = args
                    .first()
                    .map(|v| self.to_string(v))
                    .unwrap_or_else(|| ",".to_string());

                let mut parts = Vec::with_capacity(len);
                for i in 0..len {
                    let v = obj.get(&i.to_string()).cloned().unwrap_or(Value::Undefined);
                    parts.push(match v {
                        Value::Null | Value::Undefined => String::new(),
                        _ => self.to_string(&v),
                    });
                }
                Ok(Value::String(parts.join(&separator)))
            }
            BuiltinId::ArrayIndexOf => {
                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let search = args.first().cloned().unwrap_or(Value::Undefined);
                let start = args.get(1).map(|v| self.to_number(v) as usize).unwrap_or(0);

                for i in start..len {
                    if let Some(v) = obj.get(&i.to_string())
                        && self.strict_equality(&search, v)
                    {
                        return Ok(Value::Number(i as f64));
                    }
                }
                Ok(Value::Number(-1.0))
            }
            BuiltinId::ArrayLastIndexOf => {
                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                if len == 0 {
                    return Ok(Value::Number(-1.0));
                }

                let search = args.first().cloned().unwrap_or(Value::Undefined);
                let start = args
                    .get(1)
                    .map(|v| (self.to_number(v) as usize).min(len - 1))
                    .unwrap_or(len - 1);

                for i in (0..=start).rev() {
                    if let Some(v) = obj.get(&i.to_string())
                        && self.strict_equality(&search, v)
                    {
                        return Ok(Value::Number(i as f64));
                    }
                }
                Ok(Value::Number(-1.0))
            }
            BuiltinId::ArraySlice => {
                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as i64),
                        _ => None,
                    })
                    .unwrap_or(0);

                let start = args.first().map(|v| self.to_number(v) as i64).unwrap_or(0);
                let end = args.get(1).map(|v| self.to_number(v) as i64).unwrap_or(len);

                let start = if start < 0 {
                    (len + start).max(0)
                } else {
                    start.min(len)
                } as usize;
                let end = if end < 0 {
                    (len + end).max(0)
                } else {
                    end.min(len)
                } as usize;

                // Create new array
                let mut new_obj = Object::new();
                let mut new_len = 0;
                for i in start..end {
                    if let Some(v) = obj.get(&i.to_string()).cloned() {
                        new_obj.set(new_len.to_string(), v);
                    }
                    new_len += 1;
                }
                new_obj.set("length".to_string(), Value::Number(new_len as f64));

                let new_idx = self.objects.len();
                self.objects.push(new_obj);
                Ok(Value::Object(new_idx))
            }
            BuiltinId::ArrayReverse => {
                let obj = self
                    .objects
                    .get_mut(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let mut i = 0;
                let mut j = len.saturating_sub(1);
                while i < j {
                    let a = obj.get(&i.to_string()).cloned();
                    let b = obj.get(&j.to_string()).cloned();
                    if let Some(v) = b {
                        obj.set(i.to_string(), v);
                    } else {
                        obj.delete(&i.to_string());
                    }
                    if let Some(v) = a {
                        obj.set(j.to_string(), v);
                    } else {
                        obj.delete(&j.to_string());
                    }
                    i += 1;
                    j = j.saturating_sub(1);
                }
                Ok(this.clone())
            }
            BuiltinId::ArrayForEach => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                // Collect elements first to avoid borrow issues
                let elements: Vec<(usize, Value)> = (0..len)
                    .filter_map(|i| obj.get(&i.to_string()).cloned().map(|v| (i, v)))
                    .collect();

                for (i, elem) in elements {
                    self.call_function_value(
                        &callback,
                        &this_arg,
                        &[elem, Value::Number(i as f64), this.clone()],
                    )?;
                }
                Ok(Value::Undefined)
            }
            BuiltinId::ArrayMap => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                // Collect elements first
                let elements: Vec<(usize, Value)> = (0..len)
                    .map(|i| {
                        (
                            i,
                            obj.get(&i.to_string()).cloned().unwrap_or(Value::Undefined),
                        )
                    })
                    .collect();

                // Create result array
                let mut result_obj = Object::new();
                for (i, elem) in elements {
                    let mapped = self.call_function_value(
                        &callback,
                        &this_arg,
                        &[elem, Value::Number(i as f64), this.clone()],
                    )?;
                    result_obj.set(i.to_string(), mapped);
                }
                result_obj.set("length".to_string(), Value::Number(len as f64));

                let result_idx = self.objects.len();
                self.objects.push(result_obj);
                Ok(Value::Object(result_idx))
            }
            BuiltinId::ArrayFilter => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                // Collect elements first
                let elements: Vec<(usize, Value)> = (0..len)
                    .filter_map(|i| obj.get(&i.to_string()).cloned().map(|v| (i, v)))
                    .collect();

                // Create result array
                let mut result_obj = Object::new();
                let mut result_len = 0;
                for (i, elem) in elements {
                    let result = self.call_function_value(
                        &callback,
                        &this_arg,
                        &[elem.clone(), Value::Number(i as f64), this.clone()],
                    )?;
                    if result.to_boolean() {
                        result_obj.set(result_len.to_string(), elem);
                        result_len += 1;
                    }
                }
                result_obj.set("length".to_string(), Value::Number(result_len as f64));

                let result_idx = self.objects.len();
                self.objects.push(result_obj);
                Ok(Value::Object(result_idx))
            }
            BuiltinId::ArrayEvery => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let elements: Vec<(usize, Value)> = (0..len)
                    .filter_map(|i| obj.get(&i.to_string()).cloned().map(|v| (i, v)))
                    .collect();

                for (i, elem) in elements {
                    let result = self.call_function_value(
                        &callback,
                        &this_arg,
                        &[elem, Value::Number(i as f64), this.clone()],
                    )?;
                    if !result.to_boolean() {
                        return Ok(Value::Boolean(false));
                    }
                }
                Ok(Value::Boolean(true))
            }
            BuiltinId::ArraySome => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let elements: Vec<(usize, Value)> = (0..len)
                    .filter_map(|i| obj.get(&i.to_string()).cloned().map(|v| (i, v)))
                    .collect();

                for (i, elem) in elements {
                    let result = self.call_function_value(
                        &callback,
                        &this_arg,
                        &[elem, Value::Number(i as f64), this.clone()],
                    )?;
                    if result.to_boolean() {
                        return Ok(Value::Boolean(true));
                    }
                }
                Ok(Value::Boolean(false))
            }
            BuiltinId::ArrayReduce => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let initial_value = args.get(1).cloned();

                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let elements: Vec<(usize, Value)> = (0..len)
                    .filter_map(|i| obj.get(&i.to_string()).cloned().map(|v| (i, v)))
                    .collect();

                if elements.is_empty() && initial_value.is_none() {
                    return Err(Error::TypeError(
                        "Reduce of empty array with no initial value".into(),
                    ));
                }

                let mut iter = elements.into_iter();
                let mut accumulator = if let Some(init) = initial_value {
                    init
                } else {
                    iter.next().map(|(_, v)| v).unwrap_or(Value::Undefined)
                };

                for (i, elem) in iter {
                    accumulator = self.call_function_value(
                        &callback,
                        &Value::Undefined,
                        &[accumulator, elem, Value::Number(i as f64), this.clone()],
                    )?;
                }
                Ok(accumulator)
            }
            BuiltinId::ArrayReduceRight => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let initial_value = args.get(1).cloned();

                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid array".into()))?;
                let len = obj
                    .get("length")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                let elements: Vec<(usize, Value)> = (0..len)
                    .rev()
                    .filter_map(|i| obj.get(&i.to_string()).cloned().map(|v| (i, v)))
                    .collect();

                if elements.is_empty() && initial_value.is_none() {
                    return Err(Error::TypeError(
                        "Reduce of empty array with no initial value".into(),
                    ));
                }

                let mut iter = elements.into_iter();
                let mut accumulator = if let Some(init) = initial_value {
                    init
                } else {
                    iter.next().map(|(_, v)| v).unwrap_or(Value::Undefined)
                };

                for (i, elem) in iter {
                    accumulator = self.call_function_value(
                        &callback,
                        &Value::Undefined,
                        &[accumulator, elem, Value::Number(i as f64), this.clone()],
                    )?;
                }
                Ok(accumulator)
            }
            // For methods we haven't implemented yet, return undefined
            _ => Ok(Value::Undefined),
        }
    }

    /// Call a function value with given this and arguments.
    fn call_function_value(
        &mut self,
        func: &Value,
        this_value: &Value,
        args: &[Value],
    ) -> Result<Value, Error> {
        match func {
            Value::Function(func_idx) => {
                if *func_idx >= self.functions.len() {
                    return Err(Error::TypeError("Invalid function index".into()));
                }
                let func = self.functions[*func_idx].clone();
                let mut locals = vec![Value::Undefined; func.local_count.max(func.params.len())];
                for (i, arg) in args.iter().enumerate() {
                    if i < locals.len() {
                        locals[i] = arg.clone();
                    }
                }
                let frame = CallFrame {
                    return_ip: self.ip,
                    stack_base: self.stack.len(),
                    locals,
                    func_idx: Some(*func_idx),
                    bytecode_idx: 0,
                    this_value: if func.is_arrow {
                        self.this_value.clone()
                    } else {
                        this_value.clone()
                    },
                };
                self.call_stack.push(frame);
                let saved_ip = self.ip;
                let result = self.execute_function(&func)?;
                self.ip = saved_ip;
                self.call_stack.pop();
                Ok(result)
            }
            Value::NativeFunction(builtin_id) => {
                if let Some(id) = BuiltinId::from_u16(*builtin_id) {
                    call_builtin(id, this_value, args)
                } else {
                    Err(Error::TypeError("Invalid native function".into()))
                }
            }
            _ => Err(Error::TypeError(format!(
                "{} is not a function",
                func.type_of()
            ))),
        }
    }

    fn set_property(&mut self, obj: Value, key: &Value, value: Value) {
        let key_str = self.to_string(key);

        if let Value::Object(idx) = obj
            && let Some(object) = self.objects.get_mut(idx)
        {
            object.set(key_str, value);
        }
    }

    fn has_property(&self, obj: &Value, key: &Value) -> bool {
        let key_str = self.to_string(key);

        match obj {
            Value::Object(idx) => {
                if let Some(object) = self.objects.get(*idx) {
                    object.has(&key_str)
                } else {
                    false
                }
            }
            Value::String(s) => {
                key_str == "length"
                    || key_str
                        .parse::<usize>()
                        .map(|i| i < s.len())
                        .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Check if a builtin ID is an Object static method.
    fn is_object_static_method(&self, id: BuiltinId) -> bool {
        matches!(
            id,
            BuiltinId::ObjectKeys
                | BuiltinId::ObjectValues
                | BuiltinId::ObjectEntries
                | BuiltinId::ObjectCreate
                | BuiltinId::ObjectGetPrototypeOf
        )
    }

    /// Call an Object static method with access to the object heap.
    fn call_object_static_method(&mut self, id: BuiltinId, args: &[Value]) -> Result<Value, Error> {
        match id {
            BuiltinId::ObjectKeys => {
                let obj_arg = args.first().cloned().unwrap_or(Value::Undefined);
                let idx = match obj_arg {
                    Value::Object(idx) => idx,
                    _ => return Ok(Value::Object(self.create_empty_array())),
                };
                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid object".into()))?;

                // Get all enumerable own property keys
                let keys: Vec<String> = obj.properties.keys().cloned().collect();

                // Create result array
                let mut result_obj = Object::new();
                for (i, key) in keys.iter().enumerate() {
                    result_obj.set(i.to_string(), Value::String(key.clone()));
                }
                result_obj.set("length".to_string(), Value::Number(keys.len() as f64));

                let result_idx = self.objects.len();
                self.objects.push(result_obj);
                Ok(Value::Object(result_idx))
            }
            BuiltinId::ObjectValues => {
                let obj_arg = args.first().cloned().unwrap_or(Value::Undefined);
                let idx = match obj_arg {
                    Value::Object(idx) => idx,
                    _ => return Ok(Value::Object(self.create_empty_array())),
                };
                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid object".into()))?;

                let values: Vec<Value> = obj.properties.values().map(|p| p.value.clone()).collect();

                let mut result_obj = Object::new();
                for (i, value) in values.iter().enumerate() {
                    result_obj.set(i.to_string(), value.clone());
                }
                result_obj.set("length".to_string(), Value::Number(values.len() as f64));

                let result_idx = self.objects.len();
                self.objects.push(result_obj);
                Ok(Value::Object(result_idx))
            }
            BuiltinId::ObjectEntries => {
                let obj_arg = args.first().cloned().unwrap_or(Value::Undefined);
                let idx = match obj_arg {
                    Value::Object(idx) => idx,
                    _ => return Ok(Value::Object(self.create_empty_array())),
                };
                let obj = self
                    .objects
                    .get(idx)
                    .ok_or_else(|| Error::TypeError("Invalid object".into()))?;

                let entries: Vec<(String, Value)> = obj
                    .properties
                    .iter()
                    .map(|(k, p)| (k.clone(), p.value.clone()))
                    .collect();

                // Create result array of [key, value] pairs
                let mut result_obj = Object::new();
                let mut entry_objects = Vec::new();
                for (key, value) in entries.iter() {
                    let mut pair_obj = Object::new();
                    pair_obj.set("0".to_string(), Value::String(key.clone()));
                    pair_obj.set("1".to_string(), value.clone());
                    pair_obj.set("length".to_string(), Value::Number(2.0));
                    entry_objects.push(pair_obj);
                }

                // Store entry pairs in objects heap
                let start_idx = self.objects.len();
                for pair in entry_objects {
                    self.objects.push(pair);
                }

                // Build result array referencing the pairs
                for i in 0..entries.len() {
                    result_obj.set(i.to_string(), Value::Object(start_idx + i));
                }
                result_obj.set("length".to_string(), Value::Number(entries.len() as f64));

                let result_idx = self.objects.len();
                self.objects.push(result_obj);
                Ok(Value::Object(result_idx))
            }
            BuiltinId::ObjectCreate => {
                let proto = args.first().cloned().unwrap_or(Value::Null);
                // Simplified: just create an empty object (proper prototype chain not implemented)
                let _ = proto; // Prototype support would go here
                let obj = Object::new();
                let idx = self.objects.len();
                self.objects.push(obj);
                Ok(Value::Object(idx))
            }
            BuiltinId::ObjectGetPrototypeOf => {
                // Simplified: return null (proper prototype chain not implemented)
                Ok(Value::Null)
            }
            _ => Ok(Value::Undefined),
        }
    }

    /// Create an empty array and return its index.
    fn create_empty_array(&mut self) -> usize {
        let mut arr = Object::new();
        arr.set("length".to_string(), Value::Number(0.0));
        let idx = self.objects.len();
        self.objects.push(arr);
        idx
    }

    /// Check if a builtin ID is a JSON method.
    fn is_json_method(&self, id: BuiltinId) -> bool {
        matches!(id, BuiltinId::JsonParse | BuiltinId::JsonStringify)
    }

    /// Call a JSON method with access to the object heap.
    fn call_json_method(&mut self, id: BuiltinId, args: &[Value]) -> Result<Value, Error> {
        match id {
            BuiltinId::JsonParse => {
                // Parse the JSON string
                let parsed = json_parse(args)?;
                // Convert Value::Array/ParsedObject to heap objects
                self.json_to_value(parsed)
            }
            BuiltinId::JsonStringify => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                // args[1] is replacer (ignored for now), args[2] is space
                let space = self.parse_stringify_space(args.get(2));
                let mut seen = Vec::new();
                if space.is_empty() {
                    self.value_to_json(&value, &mut seen)
                } else {
                    self.value_to_json_formatted(&value, &mut seen, &space, 0)
                }
            }
            _ => Ok(Value::Undefined),
        }
    }

    /// Parse the space parameter for JSON.stringify.
    /// Returns the indent string: empty for no formatting, or spaces/string up to 10 chars.
    fn parse_stringify_space(&self, space_arg: Option<&Value>) -> String {
        match space_arg {
            Some(Value::Number(n)) => {
                let spaces = (*n as i32).clamp(0, 10) as usize;
                " ".repeat(spaces)
            }
            Some(Value::String(s)) => {
                // Truncate to max 10 characters
                s.chars().take(10).collect()
            }
            _ => String::new(),
        }
    }

    /// Convert a parsed JSON value (with Value::Array/ParsedObject) to heap objects.
    fn json_to_value(&mut self, parsed: Value) -> Result<Value, Error> {
        match parsed {
            Value::Array(elements) => {
                // Create a new array object on the heap
                let mut obj = Object::new();
                for (i, elem) in elements.into_iter().enumerate() {
                    let converted = self.json_to_value(elem)?;
                    obj.set(i.to_string(), converted);
                }
                obj.set(
                    "length".to_string(),
                    Value::Number(obj.properties.len() as f64 - 1.0),
                );
                // Fix length calculation - count numeric indices
                let len = obj
                    .properties
                    .keys()
                    .filter(|k| k.parse::<usize>().is_ok())
                    .count();
                obj.set("length".to_string(), Value::Number(len as f64));
                let idx = self.objects.len();
                self.objects.push(obj);
                Ok(Value::Object(idx))
            }
            Value::ParsedObject(pairs) => {
                // Create a new object on the heap
                let mut obj = Object::new();
                for (key, val) in pairs {
                    let converted = self.json_to_value(val)?;
                    obj.set(key, converted);
                }
                let idx = self.objects.len();
                self.objects.push(obj);
                Ok(Value::Object(idx))
            }
            // Primitives pass through unchanged
            other => Ok(other),
        }
    }

    /// Convert a value to its JSON string representation with cycle detection.
    fn value_to_json(&self, value: &Value, seen: &mut Vec<usize>) -> Result<Value, Error> {
        match value {
            Value::Null => Ok(Value::String("null".into())),
            Value::Boolean(b) => Ok(Value::String(if *b { "true" } else { "false" }.into())),
            Value::Number(n) => {
                if n.is_nan() || n.is_infinite() {
                    Ok(Value::String("null".into()))
                } else if n.fract() == 0.0 && n.abs() < 1e15 {
                    Ok(Value::String(format!("{}", *n as i64)))
                } else {
                    Ok(Value::String(format!("{}", n)))
                }
            }
            Value::String(s) => Ok(Value::String(self.escape_json_string(s))),
            Value::Undefined => Ok(Value::Undefined),
            Value::Object(idx) => {
                // Check for cycles
                if seen.contains(idx) {
                    return Err(Error::TypeError(
                        "Converting circular structure to JSON".into(),
                    ));
                }
                seen.push(*idx);

                let obj = self
                    .objects
                    .get(*idx)
                    .ok_or_else(|| Error::TypeError("Invalid object".into()))?;

                // Check if it's an array (has numeric length property)
                let is_array = obj
                    .get("length")
                    .map(|v| matches!(v, Value::Number(_)))
                    .unwrap_or(false);

                let result = if is_array {
                    self.array_to_json(*idx, seen)?
                } else {
                    self.object_to_json(*idx, seen)?
                };

                seen.pop();
                Ok(Value::String(result))
            }
            Value::Function(_) | Value::NativeFunction(_) => Ok(Value::Undefined),
            Value::Array(arr) => {
                // Handle inline arrays (from parsed JSON before heap conversion)
                let mut result = String::from("[");
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }
                    match self.value_to_json(item, seen)? {
                        Value::Undefined => result.push_str("null"),
                        Value::String(s) => result.push_str(&s),
                        _ => result.push_str("null"),
                    }
                }
                result.push(']');
                Ok(Value::String(result))
            }
            Value::ParsedObject(pairs) => {
                // Handle inline objects (from parsed JSON before heap conversion)
                let mut result = String::from("{");
                let mut first = true;
                for (key, val) in pairs {
                    if matches!(val, Value::Undefined) {
                        continue;
                    }
                    if !first {
                        result.push(',');
                    }
                    first = false;
                    result.push_str(&self.escape_json_string(key));
                    result.push(':');
                    match self.value_to_json(val, seen)? {
                        Value::Undefined => result.push_str("null"),
                        Value::String(s) => result.push_str(&s),
                        _ => result.push_str("null"),
                    }
                }
                result.push('}');
                Ok(Value::String(result))
            }
            _ => Ok(Value::String("null".into())),
        }
    }

    /// Stringify an array object to JSON.
    fn array_to_json(&self, idx: usize, seen: &mut Vec<usize>) -> Result<String, Error> {
        let obj = self
            .objects
            .get(idx)
            .ok_or_else(|| Error::TypeError("Invalid array".into()))?;

        let len = obj
            .get("length")
            .and_then(|v| match v {
                Value::Number(n) => Some(*n as usize),
                _ => None,
            })
            .unwrap_or(0);

        let mut result = String::from("[");
        for i in 0..len {
            if i > 0 {
                result.push(',');
            }
            let elem = obj.get(&i.to_string()).cloned().unwrap_or(Value::Null);
            match self.value_to_json(&elem, seen)? {
                Value::Undefined => result.push_str("null"),
                Value::String(s) => result.push_str(&s),
                _ => result.push_str("null"),
            }
        }
        result.push(']');
        Ok(result)
    }

    /// Stringify an object to JSON.
    fn object_to_json(&self, idx: usize, seen: &mut Vec<usize>) -> Result<String, Error> {
        let obj = self
            .objects
            .get(idx)
            .ok_or_else(|| Error::TypeError("Invalid object".into()))?;

        let mut result = String::from("{");
        let mut first = true;

        for (key, prop) in &obj.properties {
            // Skip length property for arrays and undefined values
            if key == "length" {
                continue;
            }
            let val = &prop.value;
            if matches!(
                val,
                Value::Undefined | Value::Function(_) | Value::NativeFunction(_)
            ) {
                continue;
            }

            if !first {
                result.push(',');
            }
            first = false;

            result.push_str(&self.escape_json_string(key));
            result.push(':');

            match self.value_to_json(val, seen)? {
                Value::Undefined => {
                    // Skip undefined - but we already filtered above, so this shouldn't happen
                    // Remove the key we just added
                    continue;
                }
                Value::String(s) => result.push_str(&s),
                _ => result.push_str("null"),
            }
        }
        result.push('}');
        Ok(result)
    }

    /// Escape a string for JSON output.
    fn escape_json_string(&self, s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 2);
        result.push('"');

        for c in s.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\u{08}' => result.push_str("\\b"),
                '\u{0C}' => result.push_str("\\f"),
                c if c.is_control() => {
                    result.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => result.push(c),
            }
        }

        result.push('"');
        result
    }

    /// Convert a value to its formatted JSON string representation with cycle detection.
    fn value_to_json_formatted(
        &self,
        value: &Value,
        seen: &mut Vec<usize>,
        indent: &str,
        depth: usize,
    ) -> Result<Value, Error> {
        match value {
            Value::Null => Ok(Value::String("null".into())),
            Value::Boolean(b) => Ok(Value::String(if *b { "true" } else { "false" }.into())),
            Value::Number(n) => {
                if n.is_nan() || n.is_infinite() {
                    Ok(Value::String("null".into()))
                } else if n.fract() == 0.0 && n.abs() < 1e15 {
                    Ok(Value::String(format!("{}", *n as i64)))
                } else {
                    Ok(Value::String(format!("{}", n)))
                }
            }
            Value::String(s) => Ok(Value::String(self.escape_json_string(s))),
            Value::Undefined => Ok(Value::Undefined),
            Value::Object(idx) => {
                // Check for cycles
                if seen.contains(idx) {
                    return Err(Error::TypeError(
                        "Converting circular structure to JSON".into(),
                    ));
                }
                seen.push(*idx);

                let obj = self
                    .objects
                    .get(*idx)
                    .ok_or_else(|| Error::TypeError("Invalid object".into()))?;

                // Check if it's an array (has numeric length property)
                let is_array = obj
                    .get("length")
                    .map(|v| matches!(v, Value::Number(_)))
                    .unwrap_or(false);

                let result = if is_array {
                    self.array_to_json_formatted(*idx, seen, indent, depth)?
                } else {
                    self.object_to_json_formatted(*idx, seen, indent, depth)?
                };

                seen.pop();
                Ok(Value::String(result))
            }
            Value::Function(_) | Value::NativeFunction(_) => Ok(Value::Undefined),
            Value::Array(arr) => {
                // Handle inline arrays (from parsed JSON before heap conversion)
                if arr.is_empty() {
                    return Ok(Value::String("[]".into()));
                }
                let current_indent = indent.repeat(depth + 1);
                let closing_indent = indent.repeat(depth);
                let mut result = String::from("[\n");
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        result.push_str(",\n");
                    }
                    result.push_str(&current_indent);
                    match self.value_to_json_formatted(item, seen, indent, depth + 1)? {
                        Value::Undefined => result.push_str("null"),
                        Value::String(s) => result.push_str(&s),
                        _ => result.push_str("null"),
                    }
                }
                result.push('\n');
                result.push_str(&closing_indent);
                result.push(']');
                Ok(Value::String(result))
            }
            Value::ParsedObject(pairs) => {
                // Handle inline objects (from parsed JSON before heap conversion)
                let non_undefined: Vec<_> = pairs
                    .iter()
                    .filter(|(_, v)| !matches!(v, Value::Undefined))
                    .collect();
                if non_undefined.is_empty() {
                    return Ok(Value::String("{}".into()));
                }
                let current_indent = indent.repeat(depth + 1);
                let closing_indent = indent.repeat(depth);
                let mut result = String::from("{\n");
                let mut first = true;
                for (key, val) in &non_undefined {
                    if !first {
                        result.push_str(",\n");
                    }
                    first = false;
                    result.push_str(&current_indent);
                    result.push_str(&self.escape_json_string(key));
                    result.push_str(": ");
                    match self.value_to_json_formatted(val, seen, indent, depth + 1)? {
                        Value::Undefined => result.push_str("null"),
                        Value::String(s) => result.push_str(&s),
                        _ => result.push_str("null"),
                    }
                }
                result.push('\n');
                result.push_str(&closing_indent);
                result.push('}');
                Ok(Value::String(result))
            }
            _ => Ok(Value::String("null".into())),
        }
    }

    /// Stringify an array object to formatted JSON.
    fn array_to_json_formatted(
        &self,
        idx: usize,
        seen: &mut Vec<usize>,
        indent: &str,
        depth: usize,
    ) -> Result<String, Error> {
        let obj = self
            .objects
            .get(idx)
            .ok_or_else(|| Error::TypeError("Invalid array".into()))?;

        let len = obj
            .get("length")
            .and_then(|v| match v {
                Value::Number(n) => Some(*n as usize),
                _ => None,
            })
            .unwrap_or(0);

        if len == 0 {
            return Ok("[]".into());
        }

        let current_indent = indent.repeat(depth + 1);
        let closing_indent = indent.repeat(depth);
        let mut result = String::from("[\n");

        for i in 0..len {
            if i > 0 {
                result.push_str(",\n");
            }
            result.push_str(&current_indent);
            let elem = obj.get(&i.to_string()).cloned().unwrap_or(Value::Null);
            match self.value_to_json_formatted(&elem, seen, indent, depth + 1)? {
                Value::Undefined => result.push_str("null"),
                Value::String(s) => result.push_str(&s),
                _ => result.push_str("null"),
            }
        }
        result.push('\n');
        result.push_str(&closing_indent);
        result.push(']');
        Ok(result)
    }

    /// Stringify an object to formatted JSON.
    fn object_to_json_formatted(
        &self,
        idx: usize,
        seen: &mut Vec<usize>,
        indent: &str,
        depth: usize,
    ) -> Result<String, Error> {
        let obj = self
            .objects
            .get(idx)
            .ok_or_else(|| Error::TypeError("Invalid object".into()))?;

        // Filter out non-serializable properties
        let serializable: Vec<_> = obj
            .properties
            .iter()
            .filter(|(key, prop)| {
                *key != "length"
                    && !matches!(
                        &prop.value,
                        Value::Undefined | Value::Function(_) | Value::NativeFunction(_)
                    )
            })
            .collect();

        if serializable.is_empty() {
            return Ok("{}".into());
        }

        let current_indent = indent.repeat(depth + 1);
        let closing_indent = indent.repeat(depth);
        let mut result = String::from("{\n");
        let mut first = true;

        for (key, prop) in &serializable {
            let val = &prop.value;
            if !first {
                result.push_str(",\n");
            }
            first = false;

            result.push_str(&current_indent);
            result.push_str(&self.escape_json_string(key));
            result.push_str(": ");

            match self.value_to_json_formatted(val, seen, indent, depth + 1)? {
                Value::Undefined => {
                    continue;
                }
                Value::String(s) => result.push_str(&s),
                _ => result.push_str("null"),
            }
        }
        result.push('\n');
        result.push_str(&closing_indent);
        result.push('}');
        Ok(result)
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::bytecode::{Instruction, Operand};

    fn make_bytecode(instructions: Vec<Instruction>, constants: Vec<Value>) -> Bytecode {
        Bytecode {
            instructions,
            constants,
            functions: Vec::new(),
        }
    }

    #[test]
    fn test_arithmetic() {
        let mut vm = VM::new();
        let bytecode = make_bytecode(
            vec![
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(0)),
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(1)),
                Instruction::simple(OpCode::Add),
                Instruction::simple(OpCode::Halt),
            ],
            vec![Value::Number(2.0), Value::Number(3.0)],
        );

        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_string_concat() {
        let mut vm = VM::new();
        let bytecode = make_bytecode(
            vec![
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(0)),
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(1)),
                Instruction::simple(OpCode::Add),
                Instruction::simple(OpCode::Halt),
            ],
            vec![
                Value::String("hello".to_string()),
                Value::String(" world".to_string()),
            ],
        );

        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_comparison() {
        let mut vm = VM::new();
        let bytecode = make_bytecode(
            vec![
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(0)),
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(1)),
                Instruction::simple(OpCode::Lt),
                Instruction::simple(OpCode::Halt),
            ],
            vec![Value::Number(1.0), Value::Number(2.0)],
        );

        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_jump() {
        let mut vm = VM::new();
        let bytecode = make_bytecode(
            vec![
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(0)), // 0
                Instruction::with_operand(OpCode::Jump, Operand::Jump(2)), // 1 -> skip to 4
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(1)), // 2 (skipped)
                Instruction::simple(OpCode::Add),                          // 3 (skipped)
                Instruction::simple(OpCode::Halt),                         // 4
            ],
            vec![Value::Number(42.0), Value::Number(100.0)],
        );

        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_global_variables() {
        let mut vm = VM::new();
        let bytecode = make_bytecode(
            vec![
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(1)), // value
                Instruction::with_operand(OpCode::StoreGlobal, Operand::Constant(0)), // name
                Instruction::with_operand(OpCode::LoadGlobal, Operand::Constant(0)), // load it back
                Instruction::simple(OpCode::Halt),
            ],
            vec![Value::String("x".to_string()), Value::Number(42.0)],
        );

        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_object_operations() {
        let mut vm = VM::new();
        let bytecode = make_bytecode(
            vec![
                Instruction::simple(OpCode::NewObject), // create {}
                Instruction::simple(OpCode::Dup),       // dup object
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(0)), // key "a"
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(1)), // value 42
                Instruction::simple(OpCode::SetProperty), // obj.a = 42
                Instruction::simple(OpCode::Pop),       // pop result
                Instruction::with_operand(OpCode::LoadConst, Operand::Constant(0)), // key "a"
                Instruction::simple(OpCode::GetProperty), // get obj.a
                Instruction::simple(OpCode::Halt),
            ],
            vec![Value::String("a".to_string()), Value::Number(42.0)],
        );

        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }
}
