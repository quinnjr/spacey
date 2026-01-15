//! The bytecode interpreter.

use crate::Error;
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
}

impl VM {
    /// Creates a new VM.
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ip: 0,
            globals: FxHashMap::default(),
            call_stack: Vec::new(),
            objects: Vec::new(),
            this_value: Value::Undefined,
        }
    }

    /// Executes bytecode and returns the result.
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value, Error> {
        self.ip = 0;
        self.stack.clear();

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
                    if let Some(Operand::ArgCount(_argc)) = &instruction.operand {
                        // Simplified: just pop the function and args, push undefined
                        // A real implementation would handle function calls properly
                        self.stack.push(Value::Undefined);
                    }
                }

                OpCode::Return => {
                    let return_value = self.pop().unwrap_or(Value::Undefined);

                    if let Some(frame) = self.call_stack.pop() {
                        // Restore state
                        self.ip = frame.return_ip;
                        self.stack.truncate(frame.stack_base);
                        self.stack.push(return_value);
                    } else {
                        // Return from top-level
                        return Ok(return_value);
                    }
                }

                OpCode::Closure => {
                    // Simplified: push undefined
                    self.stack.push(Value::Undefined);
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
            Value::Symbol(_) | Value::BigInt(_) | Value::Object(_) => f64::NAN,
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
            Value::Object(_) => "[object Object]".to_string(),
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
                    object.get(&key_str).cloned().unwrap_or(Value::Undefined)
                } else {
                    Value::Undefined
                }
            }
            Value::String(s) => {
                // String indexing
                if key_str == "length" {
                    Value::Number(s.len() as f64)
                } else if let Ok(idx) = key_str.parse::<usize>() {
                    s.chars()
                        .nth(idx)
                        .map(|c| Value::String(c.to_string()))
                        .unwrap_or(Value::Undefined)
                } else {
                    Value::Undefined
                }
            }
            _ => Value::Undefined,
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
