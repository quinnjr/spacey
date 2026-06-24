//! Bytecode compiler for JavaScript.
//!
//! Transforms AST into bytecode that can be executed by the VM.

pub mod bytecode;
mod codegen;

pub use bytecode::{Bytecode, Instruction, OpCode, Operand};
pub use codegen::Compiler;
