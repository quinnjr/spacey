//! Bytecode definitions.

use crate::runtime::value::Value;

/// A compiled bytecode chunk.
#[derive(Debug, Clone, Default)]
pub struct Bytecode {
    /// The instructions
    pub instructions: Vec<Instruction>,
    /// The constant pool
    pub constants: Vec<Value>,
}

impl Bytecode {
    /// Creates a new empty bytecode chunk.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an instruction and returns its index.
    pub fn emit(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }

    /// Adds a constant and returns its index.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        let index = self.constants.len();
        self.constants.push(value);
        index as u16
    }
}

/// A single bytecode instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    /// The operation code
    pub opcode: OpCode,
    /// Optional operand
    pub operand: Option<Operand>,
}

impl Instruction {
    /// Creates a new instruction with no operand.
    pub fn simple(opcode: OpCode) -> Self {
        Self {
            opcode,
            operand: None,
        }
    }

    /// Creates a new instruction with an operand.
    pub fn with_operand(opcode: OpCode, operand: Operand) -> Self {
        Self {
            opcode,
            operand: Some(operand),
        }
    }
}

/// Instruction operands.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Constant pool index
    Constant(u16),
    /// Local variable index
    Local(u16),
    /// Jump offset
    Jump(i32),
    /// Number of arguments
    ArgCount(u8),
    /// Property name index in constant pool
    Property(u16),
}

/// Operation codes for the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    /// Push a constant onto the stack
    LoadConst,
    /// Push undefined
    LoadUndefined,
    /// Push null
    LoadNull,
    /// Push true
    LoadTrue,
    /// Push false
    LoadFalse,
    /// Pop the top value
    Pop,
    /// Duplicate the top value
    Dup,

    // Arithmetic operations
    /// Add top two values
    Add,
    /// Subtract
    Sub,
    /// Multiply
    Mul,
    /// Divide
    Div,
    /// Modulo
    Mod,
    /// Exponentiation
    Pow,
    /// Negate (unary minus)
    Neg,

    // Comparison operations
    /// Equal (==)
    Eq,
    /// Not equal (!=)
    Ne,
    /// Strict equal (===)
    StrictEq,
    /// Strict not equal (!==)
    StrictNe,
    /// Less than
    Lt,
    /// Less than or equal
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal
    Ge,

    // Logical operations
    /// Logical NOT
    Not,

    // Bitwise operations
    /// Bitwise AND
    BitAnd,
    /// Bitwise OR
    BitOr,
    /// Bitwise XOR
    BitXor,
    /// Bitwise NOT
    BitNot,
    /// Left shift
    Shl,
    /// Signed right shift
    Shr,
    /// Unsigned right shift
    Ushr,

    // Variable operations
    /// Load a local variable
    LoadLocal,
    /// Store to a local variable
    StoreLocal,
    /// Load a global variable
    LoadGlobal,
    /// Store to a global variable
    StoreGlobal,
    /// Load from closure
    LoadUpvalue,
    /// Store to closure
    StoreUpvalue,

    // Property operations
    /// Get a property
    GetProperty,
    /// Set a property
    SetProperty,
    /// Delete a property
    DeleteProperty,

    // Control flow
    /// Unconditional jump
    Jump,
    /// Jump if false
    JumpIfFalse,
    /// Jump if true
    JumpIfTrue,
    /// Jump if not undefined (for default parameter/destructuring)
    JumpIfNotUndefined,

    // Function operations
    /// Call a function
    Call,
    /// Return from function
    Return,
    /// Create a closure
    Closure,

    // Object operations
    /// Create a new object
    NewObject,
    /// Create a new array
    NewArray,
    /// typeof operator
    TypeOf,
    /// instanceof operator
    InstanceOf,
    /// in operator
    In,

    // Special
    /// this keyword
    LoadThis,
    /// Throw an exception
    Throw,
    /// No operation
    Nop,
    /// Halt execution
    Halt,
}
