//! Code generation from AST to bytecode.
//!
//! This module contains the `Compiler` which transforms parsed JavaScript AST
//! into executable bytecode for the VM.
//!
//! ## Structure
//!
//! - `mod.rs` - Main `Compiler` struct and compilation logic
//! - `scope.rs` - Variable scope management
//! - `tests.rs` - Compiler unit tests
//!
//! ## Documentation Submodules
//!
//! - `statements` - Statement compilation documentation
//! - `expressions` - Expression compilation documentation

mod scope;

#[cfg(test)]
mod tests;

// Documentation and test submodules
pub mod expressions;
pub mod statements;

pub use scope::{Local, Scope};

use crate::Error;
use crate::ast::*;
use crate::compiler::bytecode::{Bytecode, Instruction, OpCode, Operand};
use crate::runtime::value::Value;

/// Tracks break/continue jump targets for loops and switch statements.
#[derive(Debug, Clone)]
struct LoopContext {
    /// Instruction indices that need to be patched for break
    break_jumps: Vec<usize>,
    /// Instruction indices that need to be patched for continue (loops only)
    continue_jumps: Vec<usize>,
    /// Whether this is a switch (no continue allowed)
    is_switch: bool,
    /// Optional label for this loop/switch
    label: Option<String>,
}

/// Compiles AST to bytecode.
pub struct Compiler {
    /// The bytecode being generated
    pub bytecode: Bytecode,
    /// Current scope for variable resolution
    pub scope: Scope,
    /// Stack of loop/switch contexts for break/continue
    loop_stack: Vec<LoopContext>,
    /// Pending label to apply to the next loop/switch
    pending_label: Option<String>,
    /// Variables available from enclosing function scope (for closures)
    enclosing_locals: Vec<String>,
    /// Variables captured from enclosing scope (tracked during compilation)
    captured_vars: Vec<String>,
}

impl Compiler {
    /// Creates a new compiler.
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            scope: Scope::new(),
            loop_stack: Vec::new(),
            pending_label: None,
            enclosing_locals: Vec::new(),
            captured_vars: Vec::new(),
        }
    }

    /// Creates a new compiler with enclosing scope information for closures.
    fn new_with_enclosing(enclosing_locals: Vec<String>) -> Self {
        Self {
            bytecode: Bytecode::new(),
            scope: Scope::new(),
            loop_stack: Vec::new(),
            pending_label: None,
            enclosing_locals,
            captured_vars: Vec::new(),
        }
    }

    /// Enter a loop or switch context, consuming any pending label
    fn enter_loop(&mut self, is_switch: bool) {
        let label = self.pending_label.take();
        self.loop_stack.push(LoopContext {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
            is_switch,
            label,
        });
    }

    /// Exit a loop or switch context and patch all break/continue jumps
    fn exit_loop(&mut self, break_target: usize, continue_target: Option<usize>) {
        if let Some(ctx) = self.loop_stack.pop() {
            // Patch all break jumps to jump to break_target
            for jump_idx in ctx.break_jumps {
                self.bytecode.instructions[jump_idx].operand =
                    Some(Operand::Jump(break_target as i32));
            }
            // Patch all continue jumps to jump to continue_target (if it's a loop)
            if let Some(target) = continue_target {
                for jump_idx in ctx.continue_jumps {
                    self.bytecode.instructions[jump_idx].operand =
                        Some(Operand::Jump(target as i32));
                }
            }
        }
    }

    /// Emit a break jump (returns instruction index for later patching)
    fn emit_break(&mut self) -> Result<(), Error> {
        self.emit_labeled_break(None)
    }

    /// Emit a labeled break jump
    fn emit_labeled_break(&mut self, label: Option<&str>) -> Result<(), Error> {
        let jump_idx = self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        if let Some(target_label) = label {
            // Find the loop with this label (search from innermost to outermost)
            for ctx in self.loop_stack.iter_mut().rev() {
                if ctx.label.as_deref() == Some(target_label) {
                    ctx.break_jumps.push(jump_idx);
                    return Ok(());
                }
            }
            Err(Error::SyntaxError(format!(
                "label '{}' not found",
                target_label
            )))
        } else {
            // Regular break - goes to innermost loop/switch
            if let Some(ctx) = self.loop_stack.last_mut() {
                ctx.break_jumps.push(jump_idx);
                Ok(())
            } else {
                Err(Error::SyntaxError("break outside of loop or switch".into()))
            }
        }
    }

    /// Emit a continue jump (returns instruction index for later patching)
    fn emit_continue(&mut self) -> Result<(), Error> {
        self.emit_labeled_continue(None)
    }

    /// Emit a labeled continue jump
    fn emit_labeled_continue(&mut self, label: Option<&str>) -> Result<(), Error> {
        let jump_idx = self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        if let Some(target_label) = label {
            // Find the loop with this label
            for ctx in self.loop_stack.iter_mut().rev() {
                if ctx.label.as_deref() == Some(target_label) {
                    if ctx.is_switch {
                        return Err(Error::SyntaxError("continue inside switch".into()));
                    }
                    ctx.continue_jumps.push(jump_idx);
                    return Ok(());
                }
            }
            Err(Error::SyntaxError(format!(
                "label '{}' not found",
                target_label
            )))
        } else {
            // Regular continue - goes to innermost loop (not switch)
            for ctx in self.loop_stack.iter_mut().rev() {
                if !ctx.is_switch {
                    ctx.continue_jumps.push(jump_idx);
                    return Ok(());
                }
            }
            Err(Error::SyntaxError("continue outside of loop".into()))
        }
    }

    // ========================================================================
    // Hoisting (ES3 Section 10.1.3)
    // ========================================================================

    /// Performs hoisting for a list of statements.
    /// Returns var_names (function declarations handled in-place)
    fn collect_hoisted_var_names(&self, statements: &[Statement]) -> Vec<String> {
        let mut var_names = Vec::new();
        let mut func_decls: Vec<&FunctionDeclaration> = Vec::new();

        for stmt in statements {
            self.collect_hoisted_from_statement(stmt, &mut var_names, &mut func_decls);
        }

        var_names
    }

    /// Recursively collect hoisted declarations from a statement.
    fn collect_hoisted_from_statement<'a>(
        &self,
        stmt: &'a Statement,
        var_names: &mut Vec<String>,
        func_decls: &mut Vec<&'a FunctionDeclaration>,
    ) {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                // Only hoist 'var' declarations, not 'let' or 'const'
                if decl.kind == VariableKind::Var {
                    for declarator in &decl.declarations {
                        let name = &declarator.id.name;
                        if !var_names.contains(name) {
                            var_names.push(name.clone());
                        }
                    }
                }
            }
            Statement::FunctionDeclaration(func_decl) => {
                func_decls.push(func_decl);
            }
            // Recurse into block-like statements
            Statement::Block(block) => {
                for inner_stmt in &block.body {
                    self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                }
            }
            Statement::If(if_stmt) => {
                self.collect_hoisted_from_statement(&if_stmt.consequent, var_names, func_decls);
                if let Some(alt) = &if_stmt.alternate {
                    self.collect_hoisted_from_statement(alt, var_names, func_decls);
                }
            }
            Statement::While(while_stmt) => {
                self.collect_hoisted_from_statement(&while_stmt.body, var_names, func_decls);
            }
            Statement::DoWhile(do_while) => {
                self.collect_hoisted_from_statement(&do_while.body, var_names, func_decls);
            }
            Statement::For(for_stmt) => {
                // Check init for var declarations
                if let Some(ForInit::Declaration(decl)) = &for_stmt.init
                    && decl.kind == VariableKind::Var {
                        for declarator in &decl.declarations {
                            let name = &declarator.id.name;
                            if !var_names.contains(name) {
                                var_names.push(name.clone());
                            }
                        }
                    }
                self.collect_hoisted_from_statement(&for_stmt.body, var_names, func_decls);
            }
            Statement::ForIn(for_in) => {
                if let ForInLeft::Declaration(decl) = &for_in.left
                    && decl.kind == VariableKind::Var {
                        for declarator in &decl.declarations {
                            let name = &declarator.id.name;
                            if !var_names.contains(name) {
                                var_names.push(name.clone());
                            }
                        }
                    }
                self.collect_hoisted_from_statement(&for_in.body, var_names, func_decls);
            }
            Statement::ForOf(for_of) => {
                if let ForInLeft::Declaration(decl) = &for_of.left
                    && decl.kind == VariableKind::Var {
                        for declarator in &decl.declarations {
                            let name = &declarator.id.name;
                            if !var_names.contains(name) {
                                var_names.push(name.clone());
                            }
                        }
                    }
                self.collect_hoisted_from_statement(&for_of.body, var_names, func_decls);
            }
            Statement::Switch(switch_stmt) => {
                for case in &switch_stmt.cases {
                    for inner_stmt in &case.consequent {
                        self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                    }
                }
            }
            Statement::Try(try_stmt) => {
                for inner_stmt in &try_stmt.block.body {
                    self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                }
                if let Some(handler) = &try_stmt.handler {
                    for inner_stmt in &handler.body.body {
                        self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                    }
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    for inner_stmt in &finalizer.body {
                        self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                    }
                }
            }
            Statement::With(with_stmt) => {
                self.collect_hoisted_from_statement(&with_stmt.body, var_names, func_decls);
            }
            Statement::Labeled(labeled) => {
                self.collect_hoisted_from_statement(&labeled.body, var_names, func_decls);
            }
            // Other statements don't contain nested declarations
            _ => {}
        }
    }

    /// Hoist variable and function declarations.
    fn hoist_declarations(&mut self, statements: &[Statement]) -> Result<(), Error> {
        let var_names = self.collect_hoisted_var_names(statements);
        let is_global_scope = self.scope.depth == 0;

        // Hoist var declarations as undefined
        for name in var_names {
            // Only declare if not already in scope
            if self.scope.resolve(&name).is_none() {
                self.emit(Instruction::simple(OpCode::LoadUndefined));

                if is_global_scope {
                    // At global scope, store as global so functions can access
                    let name_idx = self.bytecode.add_constant(Value::String(name));
                    self.emit(Instruction::with_operand(
                        OpCode::StoreGlobal,
                        Operand::Property(name_idx),
                    ));
                } else {
                    // Inside functions, use locals
                    let index = self.scope.declare(name, true)?;
                    self.emit(Instruction::with_operand(
                        OpCode::StoreLocal,
                        Operand::Local(index as u16),
                    ));
                    self.scope.mark_initialized(index);
                }
            }
        }

        Ok(())
    }

    // ========================================================================
    // Main Compilation Entry Point
    // ========================================================================

    /// Compiles a program AST to bytecode.
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Error> {
        // Hoist declarations first
        self.hoist_declarations(&program.body)?;

        // Compile all statements
        let len = program.body.len();
        for (i, stmt) in program.body.iter().enumerate() {
            let is_last = i == len - 1;
            self.compile_statement(stmt, is_last)?;
        }

        // Ensure there's always a value on stack
        if program.body.is_empty() {
            self.emit(Instruction::simple(OpCode::LoadUndefined));
        }

        self.emit(Instruction::simple(OpCode::Halt));

        Ok(std::mem::take(&mut self.bytecode))
    }

    // ========================================================================
    // Statement Compilation
    // ========================================================================

    fn compile_statement(&mut self, stmt: &Statement, keep_value: bool) -> Result<(), Error> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(&expr.expression)?;
                // Only pop if not the last statement (for REPL eval)
                if !keep_value {
                    self.emit(Instruction::simple(OpCode::Pop));
                }
            }
            Statement::Return(ret) => {
                if let Some(arg) = &ret.argument {
                    self.compile_expression(arg)?;
                } else {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
                self.emit(Instruction::simple(OpCode::Return));
            }
            Statement::VariableDeclaration(decl) => {
                self.compile_variable_declaration(decl)?;
                // Variable declarations produce undefined
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Block(block) => {
                let block_len = block.body.len();
                for (i, stmt) in block.body.iter().enumerate() {
                    let is_last = keep_value && i == block_len - 1;
                    self.compile_statement(stmt, is_last)?;
                }
                if block.body.is_empty() && keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::If(if_stmt) => {
                self.compile_if_statement(if_stmt, keep_value)?;
            }
            Statement::While(while_stmt) => {
                self.compile_while_statement(while_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::For(for_stmt) => {
                self.compile_for_statement(for_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::ForIn(for_in) => {
                self.compile_for_in_statement(for_in)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::ForOf(for_of) => {
                self.compile_for_of_statement(for_of)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::DoWhile(do_while) => {
                self.compile_do_while_statement(do_while)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Switch(switch_stmt) => {
                self.compile_switch_statement(switch_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Try(try_stmt) => {
                self.compile_try_statement(try_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Throw(throw_stmt) => {
                self.compile_expression(&throw_stmt.argument)?;
                self.emit(Instruction::simple(OpCode::Throw));
            }
            Statement::FunctionDeclaration(func_decl) => {
                self.compile_function_declaration(func_decl)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Empty => {
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Break => {
                self.emit_break()?;
            }
            Statement::BreakLabel(label) => {
                self.emit_labeled_break(Some(label))?;
            }
            Statement::Continue => {
                self.emit_continue()?;
            }
            Statement::ContinueLabel(label) => {
                self.emit_labeled_continue(Some(label))?;
            }
            Statement::With(with_stmt) => {
                self.compile_with_statement(with_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Labeled(labeled) => {
                self.compile_labeled_statement(labeled)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Debugger => {
                // Debugger statement - emit a nop (no-op)
                // In a full impl, would trigger debugger breakpoint
                self.emit(Instruction::simple(OpCode::Nop));
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
        }
        Ok(())
    }

    /// Compile with statement (ES3 Section 12.10).
    fn compile_with_statement(&mut self, with_stmt: &WithStatement) -> Result<(), Error> {
        // Compile the object expression
        self.compile_expression(&with_stmt.object)?;

        // In a full implementation:
        // 1. Push the object onto the scope chain
        // 2. Compile the body with the modified scope
        // 3. Pop the object from the scope chain
        //
        // For now, we just pop the object and compile the body normally
        // This is not fully compliant but avoids scope chain complexity
        self.emit(Instruction::simple(OpCode::Pop));

        // Compile the body
        self.compile_statement(&with_stmt.body, false)?;

        Ok(())
    }

    /// Compile labeled statement (ES3 Section 12.12).
    fn compile_labeled_statement(&mut self, labeled: &LabeledStatement) -> Result<(), Error> {
        // Set the pending label so the next loop/switch picks it up
        self.pending_label = Some(labeled.label.name.clone());
        // Compile the body (which may be a loop that uses the label)
        self.compile_statement(&labeled.body, false)?;
        // Clear any unused label
        self.pending_label = None;
        Ok(())
    }

    fn compile_variable_declaration(&mut self, decl: &VariableDeclaration) -> Result<(), Error> {
        let mutable = decl.kind != VariableKind::Const;
        let is_var = decl.kind == VariableKind::Var;
        let is_global_scope = self.scope.depth == 0;

        for declarator in &decl.declarations {
            // Compile initializer if present
            if let Some(init) = &declarator.init {
                self.compile_expression(init)?;
            } else if !is_var {
                // let/const without initializer gets undefined
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            } else if is_global_scope {
                // var without initializer at global scope - store undefined
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            } else {
                // var without initializer in local scope - already hoisted
                continue;
            }

            // At global scope (depth 0), use globals for var declarations
            // This allows functions to access them via LoadGlobal
            if is_global_scope && is_var {
                let name_idx = self.bytecode.add_constant(Value::String(declarator.id.name.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::StoreGlobal,
                    Operand::Property(name_idx),
                ));
            } else {
                // Use locals for let/const or var inside functions
                let index = if is_var {
                    if let Some(existing) = self.scope.resolve(&declarator.id.name) {
                        existing
                    } else {
                        self.scope.declare(declarator.id.name.clone(), mutable)?
                    }
                } else {
                    self.scope.declare(declarator.id.name.clone(), mutable)?
                };

                self.emit(Instruction::with_operand(
                    OpCode::StoreLocal,
                    Operand::Local(index as u16),
                ));
                self.scope.mark_initialized(index);
            }
        }

        Ok(())
    }

    fn compile_if_statement(
        &mut self,
        if_stmt: &IfStatement,
        keep_value: bool,
    ) -> Result<(), Error> {
        // Compile condition
        self.compile_expression(&if_stmt.test)?;

        // Jump to else/end if false
        let jump_to_else = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0), // Placeholder
        ));

        // Compile then branch
        self.compile_statement(&if_stmt.consequent, keep_value)?;

        if let Some(alternate) = &if_stmt.alternate {
            // Jump over else branch
            let jump_to_end = self.emit(Instruction::with_operand(
                OpCode::Jump,
                Operand::Jump(0), // Placeholder
            ));

            // Patch jump to else
            let else_pos = self.bytecode.instructions.len() as i32;
            self.bytecode.instructions[jump_to_else].operand = Some(Operand::Jump(else_pos));

            // Compile else branch
            self.compile_statement(alternate, keep_value)?;

            // Patch jump to end
            let end_pos = self.bytecode.instructions.len() as i32;
            self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));
        } else {
            // No else branch
            if keep_value {
                // Jump over the undefined push
                let jump_to_end =
                    self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

                // Patch jump to else (which is the undefined push)
                let else_pos = self.bytecode.instructions.len() as i32;
                self.bytecode.instructions[jump_to_else].operand = Some(Operand::Jump(else_pos));

                // Push undefined for when condition is false
                self.emit(Instruction::simple(OpCode::LoadUndefined));

                // Patch jump to end
                let end_pos = self.bytecode.instructions.len() as i32;
                self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));
            } else {
                // Patch jump to end
                let end_pos = self.bytecode.instructions.len() as i32;
                self.bytecode.instructions[jump_to_else].operand = Some(Operand::Jump(end_pos));
            }
        }

        Ok(())
    }

    fn compile_while_statement(&mut self, while_stmt: &WhileStatement) -> Result<(), Error> {
        // Enter loop context
        self.enter_loop(false);

        let loop_start = self.bytecode.instructions.len();

        // Compile condition
        self.compile_expression(&while_stmt.test)?;

        // Jump to end if false
        let jump_to_end = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0),
        ));

        // Compile body (don't keep value)
        self.compile_statement(&while_stmt.body, false)?;

        // Jump back to start
        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        // Patch jump to end
        let end_pos = self.bytecode.instructions.len();
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos as i32));

        // Exit loop context and patch break/continue
        self.exit_loop(end_pos, Some(loop_start));

        Ok(())
    }

    fn compile_for_statement(&mut self, for_stmt: &ForStatement) -> Result<(), Error> {
        // Enter loop context
        self.enter_loop(false);

        // Compile init
        if let Some(init) = &for_stmt.init {
            match init {
                ForInit::Declaration(decl) => {
                    self.compile_variable_declaration(decl)?;
                }
                ForInit::Expression(expr) => {
                    self.compile_expression(expr)?;
                    self.emit(Instruction::simple(OpCode::Pop));
                }
            }
        }

        let loop_start = self.bytecode.instructions.len();

        // Compile test
        let jump_to_end = if let Some(test) = &for_stmt.test {
            self.compile_expression(test)?;
            Some(self.emit(Instruction::with_operand(
                OpCode::JumpIfFalse,
                Operand::Jump(0),
            )))
        } else {
            None
        };

        // Compile body
        self.compile_statement(&for_stmt.body, false)?;

        // Continue target is before the update expression
        let continue_target = self.bytecode.instructions.len();

        // Compile update
        if let Some(update) = &for_stmt.update {
            self.compile_expression(update)?;
            self.emit(Instruction::simple(OpCode::Pop));
        }

        // Jump back to test
        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        // Patch jump to end
        let end_pos = self.bytecode.instructions.len();
        if let Some(jump_idx) = jump_to_end {
            self.bytecode.instructions[jump_idx].operand = Some(Operand::Jump(end_pos as i32));
        }

        // Exit loop context and patch break/continue
        self.exit_loop(end_pos, Some(continue_target));

        Ok(())
    }

    fn compile_for_in_statement(&mut self, for_in: &ForInStatement) -> Result<(), Error> {
        // Enter loop context
        self.enter_loop(false);

        // Compile the object to iterate over
        self.compile_expression(&for_in.right)?;

        // Initialize iteration (pushes keys array and index 0)
        self.emit(Instruction::simple(OpCode::ForInInit));

        let loop_start = self.bytecode.instructions.len();

        // Check if there are more keys, get next key if so
        let jump_to_end = self.emit(Instruction::with_operand(
            OpCode::ForInNext,
            Operand::Jump(0), // Placeholder - jumps to end if done
        ));

        // Store key in variable
        match &for_in.left {
            ForInLeft::Declaration(decl) => {
                // Declare variable if needed
                if !decl.declarations.is_empty() {
                    let var_name = &decl.declarations[0].id.name;
                    let mutable = decl.kind != VariableKind::Const;

                    // Check if already declared in this scope
                    if self.scope.resolve(var_name).is_none() {
                        let index = self.scope.declare(var_name.clone(), mutable)?;
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                        self.scope.mark_initialized(index);
                    } else {
                        let index = self.scope.resolve(var_name).unwrap();
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    }
                }
            }
            ForInLeft::Expression(expr) => {
                // Store to existing variable
                match expr {
                    Expression::Identifier(id) => {
                        if let Some(index) = self.scope.resolve(&id.name) {
                            self.emit(Instruction::with_operand(
                                OpCode::StoreLocal,
                                Operand::Local(index as u16),
                            ));
                        } else {
                            let name_idx =
                                self.bytecode.add_constant(Value::String(id.name.clone()));
                            self.emit(Instruction::with_operand(
                                OpCode::StoreGlobal,
                                Operand::Property(name_idx),
                            ));
                        }
                    }
                    _ => {
                        // For member expressions, would need to handle differently
                        self.emit(Instruction::simple(OpCode::Pop));
                    }
                }
            }
        }

        // Compile body
        self.compile_statement(&for_in.body, false)?;

        // Jump back to start
        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        // Patch jump to end
        let end_pos = self.bytecode.instructions.len();
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos as i32));

        // Clean up iteration state
        self.emit(Instruction::simple(OpCode::ForInDone));

        let final_end_pos = self.bytecode.instructions.len();

        // Exit loop context and patch break/continue
        self.exit_loop(final_end_pos, Some(loop_start));

        Ok(())
    }

    fn compile_for_of_statement(&mut self, for_of: &ForOfStatement) -> Result<(), Error> {
        // For-of is ES6, but we can support it with similar logic
        // For now, emit a simple stub that works like for-in
        // In a full implementation, would use Symbol.iterator

        // Compile the iterable
        self.compile_expression(&for_of.right)?;

        // Initialize iteration
        self.emit(Instruction::simple(OpCode::ForInInit));

        let loop_start = self.bytecode.instructions.len();

        let jump_to_end = self.emit(Instruction::with_operand(
            OpCode::ForInNext,
            Operand::Jump(0),
        ));

        // Store value in variable
        match &for_of.left {
            ForInLeft::Declaration(decl) => {
                if !decl.declarations.is_empty() {
                    let var_name = &decl.declarations[0].id.name;
                    let mutable = decl.kind != VariableKind::Const;

                    if self.scope.resolve(var_name).is_none() {
                        let index = self.scope.declare(var_name.clone(), mutable)?;
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                        self.scope.mark_initialized(index);
                    } else {
                        let index = self.scope.resolve(var_name).unwrap();
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    }
                }
            }
            ForInLeft::Expression(expr) => match expr {
                Expression::Identifier(id) => {
                    if let Some(index) = self.scope.resolve(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    } else {
                        let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Property(name_idx),
                        ));
                    }
                }
                _ => {
                    self.emit(Instruction::simple(OpCode::Pop));
                }
            },
        }

        self.compile_statement(&for_of.body, false)?;

        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));

        self.emit(Instruction::simple(OpCode::ForInDone));

        Ok(())
    }

    fn compile_do_while_statement(&mut self, do_while: &DoWhileStatement) -> Result<(), Error> {
        // Enter loop context
        self.enter_loop(false);

        let loop_start = self.bytecode.instructions.len();

        // Compile body first
        self.compile_statement(&do_while.body, false)?;

        // Continue target is before the condition test
        let continue_target = self.bytecode.instructions.len();

        // Compile condition
        self.compile_expression(&do_while.test)?;

        // Jump back to start if true
        self.emit(Instruction::with_operand(
            OpCode::JumpIfTrue,
            Operand::Jump(loop_start as i32),
        ));

        let end_pos = self.bytecode.instructions.len();

        // Exit loop context and patch break/continue
        self.exit_loop(end_pos, Some(continue_target));

        Ok(())
    }

    fn compile_switch_statement(&mut self, switch_stmt: &SwitchStatement) -> Result<(), Error> {
        // Enter switch context for break handling
        self.enter_loop(true);

        // Compile discriminant
        self.compile_expression(&switch_stmt.discriminant)?;

        let mut case_jumps = Vec::new();
        let mut default_case = None;

        // First pass: Generate tests and jumps
        for (i, case) in switch_stmt.cases.iter().enumerate() {
            if let Some(test) = &case.test {
                // Duplicate discriminant for comparison
                self.emit(Instruction::simple(OpCode::Dup));
                self.compile_expression(test)?;
                self.emit(Instruction::simple(OpCode::StrictEq));

                // Jump to case body if matches
                case_jumps.push((
                    i,
                    self.emit(Instruction::with_operand(
                        OpCode::JumpIfTrue,
                        Operand::Jump(0),
                    )),
                ));
            } else {
                // Default case
                default_case = Some(i);
            }
        }

        // Jump to default or end if no case matches
        let jump_to_default_or_end =
            self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        // Second pass: Generate case bodies
        let mut case_body_starts = Vec::new();
        for case in &switch_stmt.cases {
            case_body_starts.push(self.bytecode.instructions.len() as i32);
            for stmt in &case.consequent {
                self.compile_statement(stmt, false)?;
            }
        }

        // Pop discriminant
        self.emit(Instruction::simple(OpCode::Pop));

        let end_pos = self.bytecode.instructions.len();

        // Patch case jumps
        for (case_idx, jump_idx) in case_jumps {
            self.bytecode.instructions[jump_idx].operand =
                Some(Operand::Jump(case_body_starts[case_idx]));
        }

        // Patch default/end jump
        if let Some(default_idx) = default_case {
            self.bytecode.instructions[jump_to_default_or_end].operand =
                Some(Operand::Jump(case_body_starts[default_idx]));
        } else {
            self.bytecode.instructions[jump_to_default_or_end].operand =
                Some(Operand::Jump(end_pos as i32));
        }

        // Exit switch context and patch break jumps to end
        self.exit_loop(end_pos, None);

        Ok(())
    }

    fn compile_try_statement(&mut self, try_stmt: &TryStatement) -> Result<(), Error> {
        // Try-catch-finally is complex - for now, we'll just compile the blocks
        // In a full implementation, would use exception handling tables

        // Compile try block
        for stmt in &try_stmt.block.body {
            self.compile_statement(stmt, false)?;
        }

        // Compile catch block if present
        if let Some(handler) = &try_stmt.handler {
            // In a full impl, would:
            // 1. Set up exception handler
            // 2. Bind caught exception to param
            // 3. Handle the exception

            // For now, just compile the catch body
            for stmt in &handler.body.body {
                self.compile_statement(stmt, false)?;
            }
        }

        // Compile finally block if present
        if let Some(finalizer) = &try_stmt.finalizer {
            for stmt in &finalizer.body {
                self.compile_statement(stmt, false)?;
            }
        }

        Ok(())
    }

    fn compile_function_declaration(
        &mut self,
        func_decl: &FunctionDeclaration,
    ) -> Result<(), Error> {
        // Create a new compiler for the function body
        let mut func_compiler = Compiler::new();
        func_compiler.scope.begin_scope();

        // Declare parameters as locals
        let param_names: Vec<String> = func_decl.params.iter().map(|p| p.name.clone()).collect();
        for param in &param_names {
            func_compiler.scope.declare(param.clone(), true)?;
        }

        // Compile function body
        // Note: Don't use keep_value=true here - the implicit return handles the return value
        for stmt in &func_decl.body {
            func_compiler.compile_statement(stmt, false)?;
        }

        // Implicit return undefined if no explicit return
        func_compiler.emit(Instruction::simple(OpCode::LoadUndefined));
        func_compiler.emit(Instruction::simple(OpCode::Return));

        let local_count = func_compiler.scope.locals.len();
        func_compiler.scope.end_scope();

        // Create the function object
        // Note: For function declarations, we DON'T set the name as it's not a local binding
        // The name is stored in globals, not as a local inside the function
        let func_obj = crate::runtime::function::Function::new(
            None, // No local binding for function declarations
            param_names,
            std::mem::take(&mut func_compiler.bytecode),
            local_count,
        );

        // Create callable and wrap in Value
        let callable = crate::runtime::function::Callable::Function(func_obj);
        let func_value = Value::Function(std::sync::Arc::new(callable));
        let idx = self.bytecode.add_constant(func_value);

        // Load the function value
        self.emit(Instruction::with_operand(
            OpCode::LoadConst,
            Operand::Constant(idx),
        ));

        // Store function - at top level (depth 0) use global, otherwise use local
        if self.scope.depth == 0 {
            // Store as global for top-level functions
            let name_idx = self
                .bytecode
                .add_constant(Value::String(func_decl.id.name.clone()));
            self.emit(Instruction::with_operand(
                OpCode::StoreGlobal,
                Operand::Property(name_idx),
            ));
        } else {
            // Store as local for nested functions
            let local_idx = self.scope.declare(func_decl.id.name.clone(), true)?;
            self.emit(Instruction::with_operand(
                OpCode::StoreLocal,
                Operand::Local(local_idx as u16),
            ));
            self.scope.mark_initialized(local_idx);
        }

        Ok(())
    }

    // ========================================================================
    // Expression Compilation
    // ========================================================================

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), Error> {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Binary(bin) => self.compile_binary(bin),
            Expression::Unary(un) => self.compile_unary(un),
            Expression::Identifier(id) => self.compile_identifier(id),
            Expression::Assignment(assign) => self.compile_assignment(assign),
            Expression::Call(call) => self.compile_call(call),
            Expression::Member(member) => self.compile_member(member),
            Expression::This => {
                self.emit(Instruction::simple(OpCode::LoadThis));
                Ok(())
            }
            Expression::Array(arr) => self.compile_array(arr),
            Expression::Object(obj) => self.compile_object(obj),
            Expression::Conditional(cond) => self.compile_conditional(cond),
            Expression::Update(update) => self.compile_update(update),
            Expression::Sequence(seq) => self.compile_sequence(seq),
            Expression::Function(func) => self.compile_function_expr(func),
            Expression::Arrow(arrow) => self.compile_arrow(arrow),
            Expression::New(new_expr) => self.compile_new(new_expr),
        }
    }

    fn compile_identifier(&mut self, id: &Identifier) -> Result<(), Error> {
        if let Some(index) = self.scope.resolve(&id.name) {
            // Local variable
            self.emit(Instruction::with_operand(
                OpCode::LoadLocal,
                Operand::Local(index as u16),
            ));
        } else if self.enclosing_locals.contains(&id.name) {
            // Captured variable from enclosing scope
            if !self.captured_vars.contains(&id.name) {
                self.captured_vars.push(id.name.clone());
            }
            // Captured variables are injected into globals by the closure mechanism
            let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
            self.emit(Instruction::with_operand(
                OpCode::LoadGlobal,
                Operand::Property(name_idx),
            ));
        } else {
            // Global variable
            let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
            self.emit(Instruction::with_operand(
                OpCode::LoadGlobal,
                Operand::Property(name_idx),
            ));
        }
        Ok(())
    }

    fn compile_assignment(&mut self, assign: &AssignmentExpression) -> Result<(), Error> {
        // Compile the right-hand side
        self.compile_expression(&assign.right)?;

        // Handle the left-hand side
        match assign.left.as_ref() {
            Expression::Identifier(id) => {
                if let Some(index) = self.scope.resolve(&id.name) {
                    // Local variable
                    self.emit(Instruction::simple(OpCode::Dup)); // Keep value on stack
                    self.emit(Instruction::with_operand(
                        OpCode::StoreLocal,
                        Operand::Local(index as u16),
                    ));
                } else if self.enclosing_locals.contains(&id.name) {
                    // Captured variable from enclosing scope
                    if !self.captured_vars.contains(&id.name) {
                        self.captured_vars.push(id.name.clone());
                    }
                    // Captured variables are stored in globals by the closure mechanism
                    self.emit(Instruction::simple(OpCode::Dup)); // Keep value on stack
                    let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                    self.emit(Instruction::with_operand(
                        OpCode::StoreGlobal,
                        Operand::Property(name_idx),
                    ));
                } else {
                    // Global variable
                    self.emit(Instruction::simple(OpCode::Dup)); // Keep value on stack
                    let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                    self.emit(Instruction::with_operand(
                        OpCode::StoreGlobal,
                        Operand::Property(name_idx),
                    ));
                }
            }
            Expression::Member(member) => {
                // For obj.prop = value:
                // Stack order for SetProperty: [object, value] (SetProperty pops value first, then object)
                // We compile: object first, then the value is already on stack from right-hand side
                // Current stack: [value]
                // After compile object: [value, object]
                // SetProperty will pop in order: value, object - WRONG!
                //
                // Fix: Push object first, then use Swap or restructure
                // Actually, the right approach is to compile object first, then compile value
                // But we already compiled value. We need to swap the stack order.

                // Compile object
                self.compile_expression(&member.object)?;

                // Now stack is: [value, object]
                // Swap so it becomes: [object, value]
                self.emit(Instruction::simple(OpCode::Swap));

                // Compile property name
                match &member.property {
                    MemberProperty::Identifier(prop_id) => {
                        let name_idx = self
                            .bytecode
                            .add_constant(Value::String(prop_id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::SetProperty,
                            Operand::Property(name_idx),
                        ));
                    }
                    MemberProperty::Expression(prop_expr) => {
                        self.compile_expression(prop_expr)?;
                        self.emit(Instruction::simple(OpCode::SetProperty));
                    }
                }
            }
            _ => {
                return Err(Error::SyntaxError("Invalid assignment target".into()));
            }
        }

        Ok(())
    }

    fn compile_call(&mut self, call: &CallExpression) -> Result<(), Error> {
        // Compile callee
        self.compile_expression(&call.callee)?;

        // Compile arguments
        for arg in &call.arguments {
            self.compile_expression(arg)?;
        }

        // Emit call instruction
        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(call.arguments.len() as u8),
        ));

        Ok(())
    }

    fn compile_member(&mut self, member: &MemberExpression) -> Result<(), Error> {
        // Compile object
        self.compile_expression(&member.object)?;

        // Compile property access
        match &member.property {
            MemberProperty::Identifier(id) => {
                let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::GetProperty,
                    Operand::Property(name_idx),
                ));
            }
            MemberProperty::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Instruction::simple(OpCode::GetProperty));
            }
        }

        Ok(())
    }

    fn compile_array(&mut self, arr: &ArrayExpression) -> Result<(), Error> {
        // Push elements
        for elem in &arr.elements {
            if let Some(expr) = elem {
                self.compile_expression(expr)?;
            } else {
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }
        }

        // Create array
        self.emit(Instruction::with_operand(
            OpCode::NewArray,
            Operand::ArgCount(arr.elements.len() as u8),
        ));

        Ok(())
    }

    fn compile_object(&mut self, obj: &ObjectExpression) -> Result<(), Error> {
        // Create new object
        self.emit(Instruction::simple(OpCode::NewObject));

        // Set properties
        for prop in &obj.properties {
            // Duplicate object reference for each property set
            self.emit(Instruction::simple(OpCode::Dup));

            // Compile property key
            let key_name = match &prop.key {
                PropertyKey::Identifier(id) => id.name.clone(),
                PropertyKey::Literal(lit) => match lit {
                    Literal::String(s) => s.clone(),
                    Literal::Number(n) => n.to_string(),
                    _ => return Err(Error::SyntaxError("Invalid property key".into())),
                },
                PropertyKey::Computed(_) => {
                    return Err(Error::SyntaxError("Computed properties not yet supported".into()))
                }
            };

            // Compile property value
            self.compile_expression(&prop.value)?;

            // Emit SetProperty
            let key_idx = self.bytecode.add_constant(Value::String(key_name));
            self.emit(Instruction::with_operand(
                OpCode::SetProperty,
                Operand::Property(key_idx),
            ));

            // Pop the duplicated object (SetProperty leaves value on stack, we want object)
            self.emit(Instruction::simple(OpCode::Pop));
        }

        Ok(())
    }

    fn compile_conditional(&mut self, cond: &ConditionalExpression) -> Result<(), Error> {
        // Compile condition
        self.compile_expression(&cond.test)?;

        // Jump to alternate if false
        let jump_to_alternate = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0),
        ));

        // Compile consequent
        self.compile_expression(&cond.consequent)?;

        // Jump over alternate
        let jump_to_end = self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        // Patch jump to alternate
        let alternate_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_alternate].operand = Some(Operand::Jump(alternate_pos));

        // Compile alternate
        self.compile_expression(&cond.alternate)?;

        // Patch jump to end
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));

        Ok(())
    }

    fn compile_literal(&mut self, lit: &Literal) -> Result<(), Error> {
        match lit {
            Literal::Number(n) => {
                let idx = self.bytecode.add_constant(Value::Number(*n));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
            }
            Literal::String(s) => {
                let idx = self.bytecode.add_constant(Value::String(s.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
            }
            Literal::Boolean(true) => {
                self.emit(Instruction::simple(OpCode::LoadTrue));
            }
            Literal::Boolean(false) => {
                self.emit(Instruction::simple(OpCode::LoadFalse));
            }
            Literal::Null => {
                self.emit(Instruction::simple(OpCode::LoadNull));
            }
            Literal::Undefined => {
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }
            Literal::RegExp { pattern, flags } => {
                // Store regex as a string in format /pattern/flags for now
                // In a full impl, would create a RegExp object
                let regex_str = format!("/{}/{}", pattern, flags);
                let idx = self.bytecode.add_constant(Value::String(regex_str));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
            }
            Literal::BigInt(s) => {
                let idx = self.bytecode.add_constant(Value::BigInt(s.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
            }
        }
        Ok(())
    }

    fn compile_binary(&mut self, bin: &BinaryExpression) -> Result<(), Error> {
        // Handle short-circuit operators specially
        match bin.operator {
            BinaryOperator::LogicalAnd => {
                return self.compile_logical_and(bin);
            }
            BinaryOperator::LogicalOr => {
                return self.compile_logical_or(bin);
            }
            _ => {}
        }

        self.compile_expression(&bin.left)?;
        self.compile_expression(&bin.right)?;

        let opcode = match bin.operator {
            BinaryOperator::Add => OpCode::Add,
            BinaryOperator::Subtract => OpCode::Sub,
            BinaryOperator::Multiply => OpCode::Mul,
            BinaryOperator::Divide => OpCode::Div,
            BinaryOperator::Modulo => OpCode::Mod,
            BinaryOperator::LessThan => OpCode::Lt,
            BinaryOperator::LessThanEqual => OpCode::Le,
            BinaryOperator::GreaterThan => OpCode::Gt,
            BinaryOperator::GreaterThanEqual => OpCode::Ge,
            BinaryOperator::Equal => OpCode::Eq,
            BinaryOperator::NotEqual => OpCode::Ne,
            BinaryOperator::StrictEqual => OpCode::StrictEq,
            BinaryOperator::StrictNotEqual => OpCode::StrictNe,
            // Bitwise operators
            BinaryOperator::BitwiseAnd => OpCode::BitAnd,
            BinaryOperator::BitwiseOr => OpCode::BitOr,
            BinaryOperator::BitwiseXor => OpCode::BitXor,
            BinaryOperator::LeftShift => OpCode::Shl,
            BinaryOperator::RightShift => OpCode::Shr,
            BinaryOperator::UnsignedRightShift => OpCode::Ushr,
            // Object operators
            BinaryOperator::In => OpCode::In,
            BinaryOperator::InstanceOf => OpCode::InstanceOf,
            // These are handled above
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => unreachable!(),
            _ => return Err(Error::InternalError("Unsupported operator".into())),
        };

        self.emit(Instruction::simple(opcode));
        Ok(())
    }

    /// Compile logical AND with short-circuit evaluation.
    fn compile_logical_and(&mut self, bin: &BinaryExpression) -> Result<(), Error> {
        // Evaluate left side
        self.compile_expression(&bin.left)?;

        // Duplicate for the result if falsy
        self.emit(Instruction::simple(OpCode::Dup));

        // If falsy, jump to end (short-circuit)
        let jump_if_false = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0), // Placeholder
        ));

        // Pop the duplicated value (we'll use right side result)
        self.emit(Instruction::simple(OpCode::Pop));

        // Evaluate right side
        self.compile_expression(&bin.right)?;

        // Patch the jump
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_if_false].operand = Some(Operand::Jump(end_pos));

        Ok(())
    }

    /// Compile logical OR with short-circuit evaluation.
    fn compile_logical_or(&mut self, bin: &BinaryExpression) -> Result<(), Error> {
        // Evaluate left side
        self.compile_expression(&bin.left)?;

        // Duplicate for the result if truthy
        self.emit(Instruction::simple(OpCode::Dup));

        // If truthy, jump to end (short-circuit)
        let jump_if_true = self.emit(Instruction::with_operand(
            OpCode::JumpIfTrue,
            Operand::Jump(0), // Placeholder
        ));

        // Pop the duplicated value (we'll use right side result)
        self.emit(Instruction::simple(OpCode::Pop));

        // Evaluate right side
        self.compile_expression(&bin.right)?;

        // Patch the jump
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_if_true].operand = Some(Operand::Jump(end_pos));

        Ok(())
    }

    fn compile_unary(&mut self, un: &UnaryExpression) -> Result<(), Error> {
        // Handle delete specially before compiling the argument
        if let UnaryOperator::Delete = un.operator {
            return self.compile_delete(&un.argument);
        }

        self.compile_expression(&un.argument)?;

        let opcode = match un.operator {
            UnaryOperator::Minus => OpCode::Neg,
            UnaryOperator::LogicalNot => OpCode::Not,
            UnaryOperator::BitwiseNot => OpCode::BitNot,
            UnaryOperator::Typeof => OpCode::TypeOf,
            UnaryOperator::Void => {
                // void evaluates expression and returns undefined
                self.emit(Instruction::simple(OpCode::Pop));
                self.emit(Instruction::simple(OpCode::LoadUndefined));
                return Ok(());
            }
            UnaryOperator::Delete => {
                // Handled above
                unreachable!()
            }
            UnaryOperator::Plus => {
                // Unary + converts to number - we'll emit a ToNumber conversion
                // For now, just leave value on stack (it's already compiled)
                return Ok(());
            }
        };

        self.emit(Instruction::simple(opcode));
        Ok(())
    }

    /// Compile delete expression (ES3 Section 11.4.1)
    fn compile_delete(&mut self, argument: &Expression) -> Result<(), Error> {
        match argument {
            Expression::Member(member) => {
                // delete obj.prop or delete obj[expr]
                // Compile the object
                self.compile_expression(&member.object)?;

                // Emit delete instruction with property name
                match &member.property {
                    MemberProperty::Identifier(prop_id) => {
                        let name_idx = self
                            .bytecode
                            .add_constant(Value::String(prop_id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::DeleteProperty,
                            Operand::Property(name_idx),
                        ));
                    }
                    MemberProperty::Expression(prop_expr) => {
                        self.compile_expression(prop_expr)?;
                        self.emit(Instruction::simple(OpCode::DeleteProperty));
                    }
                }
            }
            Expression::Identifier(_) => {
                // delete on a simple variable - in non-strict mode, returns false for declared vars
                // For simplicity, just return true (deleting global properties)
                self.emit(Instruction::simple(OpCode::LoadTrue));
            }
            _ => {
                // delete on other expressions returns true but has no effect
                self.emit(Instruction::simple(OpCode::LoadTrue));
            }
        }
        Ok(())
    }

    /// Compile ++/-- expressions (ES3 Section 11.4.4-5, 11.3.1-2)
    fn compile_update(&mut self, update: &UpdateExpression) -> Result<(), Error> {
        // Get the variable being updated
        match update.argument.as_ref() {
            Expression::Identifier(id) => {
                if update.prefix {
                    // ++x or --x: increment/decrement first, then return new value
                    // Load current value
                    self.compile_identifier(id)?;

                    // Add/subtract 1
                    let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                    self.emit(Instruction::with_operand(
                        OpCode::LoadConst,
                        Operand::Constant(one_idx),
                    ));

                    match update.operator {
                        UpdateOperator::Increment => {
                            self.emit(Instruction::simple(OpCode::Add));
                        }
                        UpdateOperator::Decrement => {
                            self.emit(Instruction::simple(OpCode::Sub));
                        }
                    }

                    // Duplicate result (one for storage, one for return)
                    self.emit(Instruction::simple(OpCode::Dup));

                    // Store back
                    if let Some(index) = self.scope.resolve(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    } else {
                        let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Property(name_idx),
                        ));
                    }
                } else {
                    // x++ or x--: return old value, then increment/decrement
                    // Load current value
                    self.compile_identifier(id)?;

                    // Duplicate for return value
                    self.emit(Instruction::simple(OpCode::Dup));

                    // Add/subtract 1
                    let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                    self.emit(Instruction::with_operand(
                        OpCode::LoadConst,
                        Operand::Constant(one_idx),
                    ));

                    match update.operator {
                        UpdateOperator::Increment => {
                            self.emit(Instruction::simple(OpCode::Add));
                        }
                        UpdateOperator::Decrement => {
                            self.emit(Instruction::simple(OpCode::Sub));
                        }
                    }

                    // Store back
                    if let Some(index) = self.scope.resolve(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    } else {
                        let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Property(name_idx),
                        ));
                    }
                }
            }
            Expression::Member(member) => {
                // obj.prop++ or obj[key]++
                // This is more complex - need to:
                // 1. Evaluate object
                // 2. Evaluate property key
                // 3. Get current value
                // 4. Increment/decrement
                // 5. Store back
                // 6. Return old or new value based on prefix

                // For now, just compile as a simple get
                self.compile_member(member)?;

                // Add 1
                let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(one_idx),
                ));

                match update.operator {
                    UpdateOperator::Increment => {
                        self.emit(Instruction::simple(OpCode::Add));
                    }
                    UpdateOperator::Decrement => {
                        self.emit(Instruction::simple(OpCode::Sub));
                    }
                }
            }
            _ => {
                return Err(Error::SyntaxError(
                    "Invalid update expression argument".into(),
                ));
            }
        }

        Ok(())
    }

    /// Compile sequence (comma) expressions.
    fn compile_sequence(&mut self, seq: &SequenceExpression) -> Result<(), Error> {
        for (i, expr) in seq.expressions.iter().enumerate() {
            self.compile_expression(expr)?;
            // Pop all but the last result
            if i < seq.expressions.len() - 1 {
                self.emit(Instruction::simple(OpCode::Pop));
            }
        }
        Ok(())
    }

    /// Compile function expressions
    fn compile_function_expr(&mut self, func: &FunctionExpression) -> Result<(), Error> {
        // Collect enclosing scope's local variable names for closure support
        let enclosing_locals: Vec<String> = self
            .scope
            .locals
            .iter()
            .map(|l| l.name.clone())
            .collect();

        // Create a new compiler for the function body with enclosing scope info
        let mut func_compiler = Compiler::new_with_enclosing(enclosing_locals);
        func_compiler.scope.begin_scope();

        // For named function expressions, the name is available inside the function for recursion
        // We reserve slot 0 for the function reference itself
        let func_name_slot = if let Some(id) = &func.id {
            let slot = func_compiler.scope.declare(id.name.clone(), false)?;
            func_compiler.scope.mark_initialized(slot);
            Some(slot)
        } else {
            None
        };

        // Declare parameters as locals
        let param_names: Vec<String> = func.params.iter().map(|p| p.name.clone()).collect();
        for param in &param_names {
            func_compiler.scope.declare(param.clone(), true)?;
        }

        // Compile function body
        // Note: Don't use keep_value=true here - the implicit return handles the return value
        for stmt in &func.body {
            func_compiler.compile_statement(stmt, false)?;
        }

        // Implicit return undefined if no explicit return
        func_compiler.emit(Instruction::simple(OpCode::LoadUndefined));
        func_compiler.emit(Instruction::simple(OpCode::Return));

        let local_count = func_compiler.scope.locals.len();
        let captured_vars = func_compiler.captured_vars.clone();
        func_compiler.scope.end_scope();

        // Create the function object
        let func_obj = crate::runtime::function::Function::new(
            func.id.as_ref().map(|id| id.name.clone()),
            param_names,
            std::mem::take(&mut func_compiler.bytecode),
            local_count,
        );

        // Create callable and wrap in Value
        let callable = crate::runtime::function::Callable::Function(func_obj);
        let func_value = Value::Function(std::sync::Arc::new(callable));
        let idx = self.bytecode.add_constant(func_value);

        // If the function captures variables, emit code to copy locals to globals first
        // then emit MakeClosure
        if !captured_vars.is_empty() {
            // For each captured variable, copy its local value to a global
            // so MakeClosure can find it
            for var_name in &captured_vars {
                if let Some(local_idx) = self.scope.resolve(var_name) {
                    // Load the local value
                    self.emit(Instruction::with_operand(
                        OpCode::LoadLocal,
                        Operand::Local(local_idx as u16),
                    ));
                    // Store it as a global (so MakeClosure can capture it)
                    let name_idx = self.bytecode.add_constant(Value::String(var_name.clone()));
                    self.emit(Instruction::with_operand(
                        OpCode::StoreGlobal,
                        Operand::Property(name_idx),
                    ));
                }
            }

            // Load the base function
            self.emit(Instruction::with_operand(
                OpCode::LoadConst,
                Operand::Constant(idx),
            ));
            // Load the captured variable names
            let captured_names_idx = self
                .bytecode
                .add_constant(Value::String(captured_vars.join(",")));
            self.emit(Instruction::with_operand(
                OpCode::LoadConst,
                Operand::Constant(captured_names_idx),
            ));
            // Create closure at runtime
            self.emit(Instruction::simple(OpCode::MakeClosure));
        } else {
            self.emit(Instruction::with_operand(
                OpCode::LoadConst,
                Operand::Constant(idx),
            ));
        }

        // Store info about the function name slot for the VM to use
        let _ = func_name_slot; // Mark as used for now

        Ok(())
    }

    /// Compile arrow function expressions
    fn compile_arrow(&mut self, arrow: &ArrowFunctionExpression) -> Result<(), Error> {
        // Arrow functions are similar to function expressions
        let mut func_compiler = Compiler::new();
        func_compiler.scope.begin_scope();

        let param_names: Vec<String> = arrow.params.iter().map(|p| p.name.clone()).collect();
        for param in &param_names {
            func_compiler.scope.declare(param.clone(), true)?;
        }

        match &arrow.body {
            ArrowBody::Expression(expr) => {
                func_compiler.compile_expression(expr)?;
                func_compiler.emit(Instruction::simple(OpCode::Return));
            }
            ArrowBody::Block(stmts) => {
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last = i == stmts.len() - 1;
                    func_compiler.compile_statement(stmt, is_last)?;
                }
                func_compiler.emit(Instruction::simple(OpCode::LoadUndefined));
                func_compiler.emit(Instruction::simple(OpCode::Return));
            }
        }

        let local_count = func_compiler.scope.locals.len();
        func_compiler.scope.end_scope();

        // Create the function object
        let func_obj = crate::runtime::function::Function::new(
            None, // Arrow functions are always anonymous
            param_names,
            std::mem::take(&mut func_compiler.bytecode),
            local_count,
        );

        let callable = crate::runtime::function::Callable::Function(func_obj);
        let func_value = Value::Function(std::sync::Arc::new(callable));
        let idx = self.bytecode.add_constant(func_value);
        self.emit(Instruction::with_operand(
            OpCode::LoadConst,
            Operand::Constant(idx),
        ));

        Ok(())
    }

    /// Compile new expressions
    fn compile_new(&mut self, new_expr: &NewExpression) -> Result<(), Error> {
        // Compile the constructor
        self.compile_expression(&new_expr.callee)?;

        // Compile arguments
        for arg in &new_expr.arguments {
            self.compile_expression(arg)?;
        }

        // Emit new instruction (for now, just call)
        // In a full impl, would create new object with prototype chain
        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(new_expr.arguments.len() as u8),
        ));

        Ok(())
    }

    // ========================================================================
    // Utilities
    // ========================================================================

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.bytecode.emit(instruction)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

