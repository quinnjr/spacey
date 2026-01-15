//! Code generation from AST to bytecode.

use crate::Error;
use crate::ast::*;
use crate::compiler::bytecode::{Bytecode, Instruction, OpCode, Operand};
use crate::runtime::value::Value;
use rustc_hash::FxHashMap;

/// Represents a local variable in the current scope.
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: u32,
    /// Variable kind (var, let, const)
    kind: LocalKind,
    /// Whether the variable has been initialized (for TDZ)
    initialized: bool,
}

/// Kind of local variable binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalKind {
    /// var - function scoped, hoisted
    Var,
    /// let - block scoped, TDZ applies
    Let,
    /// const - block scoped, TDZ applies, immutable
    Const,
    /// Function parameter
    Parameter,
}

/// Represents a break or continue target.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LoopContext {
    /// Label for this loop (if any)
    label: Option<String>,
    /// Bytecode positions where break should jump to (to be patched)
    break_jumps: Vec<usize>,
    /// Bytecode positions where continue should jump to (to be patched)
    continue_jumps: Vec<usize>,
    /// Position to jump to for continue
    continue_target: Option<usize>,
}

/// Compiles AST to bytecode.
pub struct Compiler {
    bytecode: Bytecode,
    /// Local variables in scope
    locals: Vec<Local>,
    /// Current scope depth
    scope_depth: u32,
    /// Stack of loop contexts for break/continue
    loop_stack: Vec<LoopContext>,
    /// Global variable names -> constant pool indices
    globals: FxHashMap<String, u16>,
}

impl Compiler {
    /// Creates a new compiler.
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loop_stack: Vec::new(),
            globals: FxHashMap::default(),
        }
    }

    /// Compiles a program to bytecode.
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Error> {
        let len = program.body.len();
        for (i, item) in program.body.iter().enumerate() {
            let is_last = i == len - 1;
            self.compile_module_item(item, is_last)?;
        }
        self.emit(Instruction::simple(OpCode::Halt));
        Ok(std::mem::take(&mut self.bytecode))
    }

    fn compile_module_item(&mut self, item: &ModuleItem, is_last: bool) -> Result<(), Error> {
        match item {
            ModuleItem::Statement(stmt) => self.compile_statement_with_context(stmt, is_last),
            ModuleItem::ImportDeclaration(_) => {
                // Import declarations are handled at module loading time
                // For now, emit nothing
                Ok(())
            }
            ModuleItem::ExportDeclaration(export) => {
                match export {
                    ExportDeclaration::Declaration(stmt) => {
                        self.compile_statement(stmt)?;
                    }
                    ExportDeclaration::Default(expr) => {
                        self.compile_expression(expr)?;
                        // Store in special 'default' export slot
                        self.emit(Instruction::simple(OpCode::Pop));
                    }
                    ExportDeclaration::Named { .. } | ExportDeclaration::All { .. } => {
                        // These are resolved at module linking time
                    }
                }
                Ok(())
            }
        }
    }

    fn compile_statement_with_context(
        &mut self,
        stmt: &Statement,
        is_last: bool,
    ) -> Result<(), Error> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(&expr.expression)?;
                // Only pop if not the last statement (keep result for return value)
                if !is_last {
                    self.emit(Instruction::simple(OpCode::Pop));
                }
                Ok(())
            }
            _ => self.compile_statement(stmt),
        }
    }

    // ==================== Statement Compilation ====================

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), Error> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(&expr.expression)?;
                self.emit(Instruction::simple(OpCode::Pop));
            }
            Statement::VariableDeclaration(decl) => {
                self.compile_variable_declaration(decl)?;
            }
            Statement::FunctionDeclaration(func) => {
                self.compile_function_declaration(func)?;
            }
            Statement::ClassDeclaration(class) => {
                self.compile_class_declaration(class)?;
            }
            Statement::Block(block) => {
                self.begin_scope();
                for stmt in &block.body {
                    self.compile_statement(stmt)?;
                }
                self.end_scope();
            }
            Statement::If(if_stmt) => {
                self.compile_if_statement(if_stmt)?;
            }
            Statement::While(while_stmt) => {
                self.compile_while_statement(while_stmt)?;
            }
            Statement::DoWhile(do_while) => {
                self.compile_do_while_statement(do_while)?;
            }
            Statement::For(for_stmt) => {
                self.compile_for_statement(for_stmt)?;
            }
            Statement::ForIn(for_in) => {
                self.compile_for_in_statement(for_in)?;
            }
            Statement::ForOf(for_of) => {
                self.compile_for_of_statement(for_of)?;
            }
            Statement::Switch(switch) => {
                self.compile_switch_statement(switch)?;
            }
            Statement::Return(ret) => {
                if let Some(arg) = &ret.argument {
                    self.compile_expression(arg)?;
                } else {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
                self.emit(Instruction::simple(OpCode::Return));
            }
            Statement::Break(brk) => {
                self.compile_break(&brk.label)?;
            }
            Statement::Continue(cont) => {
                self.compile_continue(&cont.label)?;
            }
            Statement::Throw(throw) => {
                self.compile_expression(&throw.argument)?;
                self.emit(Instruction::simple(OpCode::Throw));
            }
            Statement::Try(try_stmt) => {
                self.compile_try_statement(try_stmt)?;
            }
            Statement::With(_) => {
                // 'with' is deprecated and complex, skip for now
                return Err(Error::InternalError(
                    "'with' statement not supported".into(),
                ));
            }
            Statement::Labeled(labeled) => {
                self.compile_labeled_statement(labeled)?;
            }
            Statement::Debugger => {
                // No-op for now
                self.emit(Instruction::simple(OpCode::Nop));
            }
            Statement::Empty => {
                // Nothing to do
            }
        }
        Ok(())
    }

    fn compile_variable_declaration(&mut self, decl: &VariableDeclaration) -> Result<(), Error> {
        let local_kind = match decl.kind {
            VariableKind::Var => LocalKind::Var,
            VariableKind::Let => LocalKind::Let,
            VariableKind::Const => LocalKind::Const,
        };

        for declarator in &decl.declarations {
            if let Some(init) = &declarator.init {
                self.compile_expression(init)?;
            } else {
                // const must have initializer
                if decl.kind == VariableKind::Const {
                    return Err(Error::SyntaxError(
                        "Missing initializer in const declaration".into(),
                    ));
                }
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }

            // Compile the binding pattern
            self.compile_binding_pattern(&declarator.id, local_kind)?;
        }
        Ok(())
    }

    fn compile_binding_pattern(
        &mut self,
        pattern: &BindingPattern,
        local_kind: LocalKind,
    ) -> Result<(), Error> {
        match pattern {
            BindingPattern::Identifier(id) => {
                // Simple identifier binding
                if self.scope_depth > 0 {
                    self.add_local(&id.name, local_kind);
                } else {
                    let name_idx = self.add_global_name(&id.name);
                    self.emit(Instruction::with_operand(
                        OpCode::StoreGlobal,
                        Operand::Constant(name_idx),
                    ));
                    self.emit(Instruction::simple(OpCode::Pop));
                }
            }
            BindingPattern::Array(arr_pattern) => {
                // Array destructuring
                // For now, simplified: assumes value on stack is an array
                // and emits GetProperty for each index
                for (i, element) in arr_pattern.elements.iter().enumerate() {
                    if let Some(elem) = element {
                        self.emit(Instruction::simple(OpCode::Dup));
                        let idx = self.bytecode.add_constant(Value::Number(i as f64));
                        self.emit(Instruction::with_operand(
                            OpCode::LoadConst,
                            Operand::Constant(idx),
                        ));
                        self.emit(Instruction::simple(OpCode::GetProperty));

                        // Handle default value
                        if let Some(default) = &elem.default {
                            // Check if undefined and use default
                            let skip_default = self.emit_jump(OpCode::JumpIfNotUndefined);
                            self.emit(Instruction::simple(OpCode::Pop));
                            self.compile_expression(default)?;
                            self.patch_jump(skip_default);
                        }

                        self.compile_binding_pattern(&elem.pattern, local_kind)?;
                    }
                }

                // Handle rest element
                if let Some(rest) = &arr_pattern.rest {
                    // For now, just emit a placeholder
                    // Full implementation would slice the array
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                    self.compile_binding_pattern(rest, local_kind)?;
                }

                // Pop the original array
                self.emit(Instruction::simple(OpCode::Pop));
            }
            BindingPattern::Object(obj_pattern) => {
                // Object destructuring
                for prop in &obj_pattern.properties {
                    self.emit(Instruction::simple(OpCode::Dup));

                    // Get the property
                    match &prop.key {
                        PropertyKey::Identifier(id) => {
                            let idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                            self.emit(Instruction::with_operand(
                                OpCode::LoadConst,
                                Operand::Constant(idx),
                            ));
                        }
                        PropertyKey::Literal(Literal::String(s)) => {
                            let idx = self.bytecode.add_constant(Value::String(s.clone()));
                            self.emit(Instruction::with_operand(
                                OpCode::LoadConst,
                                Operand::Constant(idx),
                            ));
                        }
                        PropertyKey::Literal(Literal::Number(n)) => {
                            let idx = self.bytecode.add_constant(Value::String(n.to_string()));
                            self.emit(Instruction::with_operand(
                                OpCode::LoadConst,
                                Operand::Constant(idx),
                            ));
                        }
                        PropertyKey::Computed(expr) => {
                            self.compile_expression(expr)?;
                        }
                        _ => {}
                    }
                    self.emit(Instruction::simple(OpCode::GetProperty));

                    // Handle default value
                    if let Some(default) = &prop.default {
                        let skip_default = self.emit_jump(OpCode::JumpIfNotUndefined);
                        self.emit(Instruction::simple(OpCode::Pop));
                        self.compile_expression(default)?;
                        self.patch_jump(skip_default);
                    }

                    self.compile_binding_pattern(&prop.value, local_kind)?;
                }

                // Handle rest element
                if let Some(rest_id) = &obj_pattern.rest {
                    // For now, just emit a placeholder
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                    self.compile_binding_pattern(
                        &BindingPattern::Identifier(rest_id.clone()),
                        local_kind,
                    )?;
                }

                // Pop the original object
                self.emit(Instruction::simple(OpCode::Pop));
            }
            BindingPattern::Rest(inner) => {
                // Rest pattern in binding - the value should already be on stack
                self.compile_binding_pattern(inner, local_kind)?;
            }
        }
        Ok(())
    }

    fn compile_function_declaration(&mut self, func: &FunctionDeclaration) -> Result<(), Error> {
        // Compile function body to a separate bytecode chunk
        // For now, we'll use a simplified approach

        // Create closure and store in variable
        let func_idx = self.bytecode.add_constant(Value::Undefined); // Placeholder
        self.emit(Instruction::with_operand(
            OpCode::LoadConst,
            Operand::Constant(func_idx),
        ));

        if self.scope_depth > 0 {
            // Function declarations are hoisted and act like var
            self.add_local(&func.id.name, LocalKind::Var);
        } else {
            let name_idx = self.add_global_name(&func.id.name);
            self.emit(Instruction::with_operand(
                OpCode::StoreGlobal,
                Operand::Constant(name_idx),
            ));
            self.emit(Instruction::simple(OpCode::Pop));
        }

        Ok(())
    }

    fn compile_class_declaration(&mut self, class: &ClassDeclaration) -> Result<(), Error> {
        // Compile the class into a constructor function with prototype methods
        // For now, this is a simplified implementation

        // Create the class constructor
        self.emit(Instruction::simple(OpCode::NewObject)); // Placeholder for class object

        // Store in variable
        if self.scope_depth > 0 {
            self.add_local(&class.id.name, LocalKind::Let);
        } else {
            let name_idx = self.add_global_name(&class.id.name);
            self.emit(Instruction::with_operand(
                OpCode::StoreGlobal,
                Operand::Constant(name_idx),
            ));
            self.emit(Instruction::simple(OpCode::Pop));
        }

        Ok(())
    }

    fn compile_class_expression(&mut self, _class: &ClassExpression) -> Result<(), Error> {
        // Compile class expression to a constructor function
        // For now, emit a placeholder
        self.emit(Instruction::simple(OpCode::NewObject));
        Ok(())
    }

    fn compile_if_statement(&mut self, if_stmt: &IfStatement) -> Result<(), Error> {
        // Compile condition
        self.compile_expression(&if_stmt.test)?;

        // Jump to else if false
        let jump_to_else = self.emit_jump(OpCode::JumpIfFalse);
        self.emit(Instruction::simple(OpCode::Pop)); // Pop condition

        // Compile then branch
        self.compile_statement(&if_stmt.consequent)?;

        if let Some(alternate) = &if_stmt.alternate {
            // Jump over else
            let jump_over_else = self.emit_jump(OpCode::Jump);

            // Patch jump to else
            self.patch_jump(jump_to_else);
            self.emit(Instruction::simple(OpCode::Pop)); // Pop condition

            // Compile else branch
            self.compile_statement(alternate)?;

            // Patch jump over else
            self.patch_jump(jump_over_else);
        } else {
            self.patch_jump(jump_to_else);
            self.emit(Instruction::simple(OpCode::Pop)); // Pop condition
        }

        Ok(())
    }

    fn compile_while_statement(&mut self, while_stmt: &WhileStatement) -> Result<(), Error> {
        let loop_start = self.bytecode.instructions.len();

        self.push_loop(None);

        // Set continue target
        self.set_continue_target(loop_start);

        // Compile condition
        self.compile_expression(&while_stmt.test)?;

        // Jump out if false
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit(Instruction::simple(OpCode::Pop)); // Pop condition

        // Compile body
        self.compile_statement(&while_stmt.body)?;

        // Jump back to start
        self.emit_loop(loop_start);

        // Patch exit jump
        self.patch_jump(exit_jump);
        self.emit(Instruction::simple(OpCode::Pop)); // Pop condition

        self.pop_loop();

        Ok(())
    }

    fn compile_do_while_statement(&mut self, do_while: &DoWhileStatement) -> Result<(), Error> {
        let loop_start = self.bytecode.instructions.len();

        self.push_loop(None);

        // Compile body first
        self.compile_statement(&do_while.body)?;

        // Set continue target (to condition check)
        let condition_start = self.bytecode.instructions.len();
        self.set_continue_target(condition_start);

        // Compile condition
        self.compile_expression(&do_while.test)?;

        // Jump back if true
        let offset = loop_start as i32 - (self.bytecode.instructions.len() as i32 + 1);
        self.emit(Instruction::with_operand(
            OpCode::JumpIfTrue,
            Operand::Jump(offset),
        ));
        self.emit(Instruction::simple(OpCode::Pop)); // Pop condition

        self.pop_loop();

        Ok(())
    }

    fn compile_for_statement(&mut self, for_stmt: &ForStatement) -> Result<(), Error> {
        self.begin_scope();

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
        self.push_loop(None);

        // Compile condition
        let mut exit_jump = None;
        if let Some(test) = &for_stmt.test {
            self.compile_expression(test)?;
            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse));
            self.emit(Instruction::simple(OpCode::Pop));
        }

        // Compile body
        self.compile_statement(&for_stmt.body)?;

        // Set continue target (to update)
        let update_start = self.bytecode.instructions.len();
        self.set_continue_target(update_start);

        // Compile update
        if let Some(update) = &for_stmt.update {
            self.compile_expression(update)?;
            self.emit(Instruction::simple(OpCode::Pop));
        }

        // Jump back to start
        self.emit_loop(loop_start);

        // Patch exit jump
        if let Some(jump) = exit_jump {
            self.patch_jump(jump);
            self.emit(Instruction::simple(OpCode::Pop));
        }

        self.pop_loop();
        self.end_scope();

        Ok(())
    }

    fn compile_for_in_statement(&mut self, _for_in: &ForInStatement) -> Result<(), Error> {
        // for-in requires iterator protocol support
        // For now, emit a placeholder
        self.emit(Instruction::simple(OpCode::Nop));
        Ok(())
    }

    fn compile_for_of_statement(&mut self, _for_of: &ForOfStatement) -> Result<(), Error> {
        // for-of requires iterator protocol support (Symbol.iterator)
        // For now, emit a placeholder
        self.emit(Instruction::simple(OpCode::Nop));
        Ok(())
    }

    fn compile_switch_statement(&mut self, switch: &SwitchStatement) -> Result<(), Error> {
        self.compile_expression(&switch.discriminant)?;

        let mut case_jumps = Vec::new();
        let mut default_jump = None;

        // First pass: emit comparisons and jumps
        for case in &switch.cases {
            if let Some(test) = &case.test {
                self.emit(Instruction::simple(OpCode::Dup));
                self.compile_expression(test)?;
                self.emit(Instruction::simple(OpCode::StrictEq));
                case_jumps.push(self.emit_jump(OpCode::JumpIfTrue));
                self.emit(Instruction::simple(OpCode::Pop)); // Pop comparison result
            } else {
                default_jump = Some(case_jumps.len());
            }
        }

        // Jump to default or end
        let end_jump = self.emit_jump(OpCode::Jump);

        // Second pass: emit case bodies
        let mut body_positions = Vec::new();
        self.push_loop(None); // For break statements

        for case in &switch.cases {
            body_positions.push(self.bytecode.instructions.len());
            self.emit(Instruction::simple(OpCode::Pop)); // Pop discriminant duplicate or comparison result
            for stmt in &case.consequent {
                self.compile_statement(stmt)?;
            }
        }

        self.pop_loop();

        // Patch case jumps
        for (i, jump) in case_jumps.into_iter().enumerate() {
            let target = body_positions[i];
            let offset = target as i32 - (jump as i32 + 1);
            self.bytecode.instructions[jump] =
                Instruction::with_operand(OpCode::JumpIfTrue, Operand::Jump(offset));
        }

        // Patch end jump
        if let Some(default_idx) = default_jump {
            let target = body_positions[default_idx];
            let offset = target as i32 - (end_jump as i32 + 1);
            self.bytecode.instructions[end_jump] =
                Instruction::with_operand(OpCode::Jump, Operand::Jump(offset));
        } else {
            self.patch_jump(end_jump);
        }

        self.emit(Instruction::simple(OpCode::Pop)); // Pop discriminant

        Ok(())
    }

    fn compile_break(&mut self, label: &Option<Identifier>) -> Result<(), Error> {
        let jump = self.emit_jump(OpCode::Jump);

        // Find the right loop context
        if let Some(label_id) = label {
            for ctx in self.loop_stack.iter_mut().rev() {
                if ctx.label.as_ref() == Some(&label_id.name) {
                    ctx.break_jumps.push(jump);
                    return Ok(());
                }
            }
            return Err(Error::SyntaxError(format!(
                "Undefined label: {}",
                label_id.name
            )));
        } else if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.break_jumps.push(jump);
        } else {
            return Err(Error::SyntaxError("break outside loop".into()));
        }

        Ok(())
    }

    fn compile_continue(&mut self, label: &Option<Identifier>) -> Result<(), Error> {
        // Find the right loop context
        if let Some(label_id) = label {
            for ctx in self.loop_stack.iter().rev() {
                if ctx.label.as_ref() == Some(&label_id.name)
                    && let Some(target) = ctx.continue_target
                {
                    self.emit_loop(target);
                    return Ok(());
                }
            }
            return Err(Error::SyntaxError(format!(
                "Undefined label: {}",
                label_id.name
            )));
        } else if let Some(ctx) = self.loop_stack.last()
            && let Some(target) = ctx.continue_target
        {
            self.emit_loop(target);
            return Ok(());
        }

        Err(Error::SyntaxError("continue outside loop".into()))
    }

    fn compile_try_statement(&mut self, _try_stmt: &TryStatement) -> Result<(), Error> {
        // Try/catch requires exception handler tables
        // For now, emit a placeholder
        self.emit(Instruction::simple(OpCode::Nop));
        Ok(())
    }

    fn compile_labeled_statement(&mut self, labeled: &LabeledStatement) -> Result<(), Error> {
        // For labeled loops, we need to set up the loop context first
        self.push_loop(Some(labeled.label.name.clone()));
        self.compile_statement(&labeled.body)?;
        self.pop_loop();
        Ok(())
    }

    // ==================== Expression Compilation ====================

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), Error> {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Identifier(id) => self.compile_identifier(id),
            Expression::This => {
                self.emit(Instruction::simple(OpCode::LoadThis));
                Ok(())
            }
            Expression::Array(arr) => self.compile_array(arr),
            Expression::Object(obj) => self.compile_object(obj),
            Expression::Binary(bin) => self.compile_binary(bin),
            Expression::Unary(un) => self.compile_unary(un),
            Expression::Assignment(assign) => self.compile_assignment(assign),
            Expression::Call(call) => self.compile_call(call),
            Expression::Member(member) => self.compile_member(member, false),
            Expression::Conditional(cond) => self.compile_conditional(cond),
            Expression::Function(func) => self.compile_function_expression(func),
            Expression::New(new_expr) => self.compile_new(new_expr),
            Expression::Update(update) => self.compile_update(update),
            Expression::Sequence(seq) => self.compile_sequence(seq),
            Expression::Arrow(arrow) => self.compile_arrow_function(arrow),
            Expression::TemplateLiteral(template) => self.compile_template_literal(template),
            Expression::TaggedTemplate(tagged) => self.compile_tagged_template(tagged),
            Expression::Class(class) => self.compile_class_expression(class),
            Expression::Super => {
                // Super requires special handling in the context of methods
                // For now, emit a placeholder
                self.emit(Instruction::simple(OpCode::LoadUndefined));
                Ok(())
            }
            Expression::Yield(yield_expr) => {
                // Yield requires generator state machine support
                // For now, emit a placeholder that evaluates the argument
                if let Some(arg) = &yield_expr.argument {
                    self.compile_expression(arg)?;
                } else {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
                Ok(())
            }
            Expression::Await(await_expr) => {
                // Await requires async/await support
                // For now, just compile the argument
                self.compile_expression(&await_expr.argument)?;
                Ok(())
            }
        }
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
            Literal::BigInt(_) | Literal::RegExp { .. } => {
                // BigInt and RegExp need more complex handling
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }
        }
        Ok(())
    }

    fn compile_identifier(&mut self, id: &Identifier) -> Result<(), Error> {
        // Check TDZ for let/const
        self.check_local_tdz(&id.name)?;

        // Check if it's a local variable
        if let Some(idx) = self.resolve_local(&id.name) {
            self.emit(Instruction::with_operand(
                OpCode::LoadLocal,
                Operand::Local(idx),
            ));
        } else {
            // Global variable
            let name_idx = self.add_global_name(&id.name);
            self.emit(Instruction::with_operand(
                OpCode::LoadGlobal,
                Operand::Constant(name_idx),
            ));
        }
        Ok(())
    }

    fn compile_array(&mut self, arr: &ArrayExpression) -> Result<(), Error> {
        let count = arr.elements.len();

        for element in &arr.elements {
            match element {
                Some(ArrayElement::Expression(expr)) => {
                    self.compile_expression(expr)?;
                }
                Some(ArrayElement::Spread(expr)) => {
                    // For now, compile spread as regular expression
                    // Full spread support would require runtime iteration
                    self.compile_expression(expr)?;
                }
                None => {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
        }

        self.emit(Instruction::with_operand(
            OpCode::NewArray,
            Operand::ArgCount(count as u8),
        ));
        Ok(())
    }

    fn compile_object(&mut self, obj: &ObjectExpression) -> Result<(), Error> {
        self.emit(Instruction::simple(OpCode::NewObject));

        for prop in &obj.properties {
            match prop {
                ObjectProperty::Property(prop) => {
                    // Duplicate object reference
                    self.emit(Instruction::simple(OpCode::Dup));

                    // Compile property key
                    match &prop.key {
                        PropertyKey::Identifier(id) => {
                            let idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                            self.emit(Instruction::with_operand(
                                OpCode::LoadConst,
                                Operand::Constant(idx),
                            ));
                        }
                        PropertyKey::Literal(Literal::String(s)) => {
                            let idx = self.bytecode.add_constant(Value::String(s.clone()));
                            self.emit(Instruction::with_operand(
                                OpCode::LoadConst,
                                Operand::Constant(idx),
                            ));
                        }
                        PropertyKey::Literal(Literal::Number(n)) => {
                            let idx = self.bytecode.add_constant(Value::String(n.to_string()));
                            self.emit(Instruction::with_operand(
                                OpCode::LoadConst,
                                Operand::Constant(idx),
                            ));
                        }
                        PropertyKey::Computed(expr) => {
                            self.compile_expression(expr)?;
                        }
                        _ => {}
                    }

                    // Compile property value
                    self.compile_expression(&prop.value)?;

                    // Set property
                    self.emit(Instruction::simple(OpCode::SetProperty));
                }
                ObjectProperty::Spread(expr) => {
                    // For spread, we need to copy all properties from the source object
                    // For now, emit a placeholder - full implementation requires runtime support
                    let _ = expr; // Suppress warning
                }
            }
        }

        Ok(())
    }

    fn compile_binary(&mut self, bin: &BinaryExpression) -> Result<(), Error> {
        // Handle short-circuit operators specially
        match bin.operator {
            BinaryOperator::LogicalAnd => {
                self.compile_expression(&bin.left)?;
                let jump = self.emit_jump(OpCode::JumpIfFalse);
                self.emit(Instruction::simple(OpCode::Pop));
                self.compile_expression(&bin.right)?;
                self.patch_jump(jump);
                return Ok(());
            }
            BinaryOperator::LogicalOr => {
                self.compile_expression(&bin.left)?;
                let jump = self.emit_jump(OpCode::JumpIfTrue);
                self.emit(Instruction::simple(OpCode::Pop));
                self.compile_expression(&bin.right)?;
                self.patch_jump(jump);
                return Ok(());
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
            BinaryOperator::Exponent => OpCode::Pow,
            BinaryOperator::LessThan => OpCode::Lt,
            BinaryOperator::LessThanEqual => OpCode::Le,
            BinaryOperator::GreaterThan => OpCode::Gt,
            BinaryOperator::GreaterThanEqual => OpCode::Ge,
            BinaryOperator::Equal => OpCode::Eq,
            BinaryOperator::NotEqual => OpCode::Ne,
            BinaryOperator::StrictEqual => OpCode::StrictEq,
            BinaryOperator::StrictNotEqual => OpCode::StrictNe,
            BinaryOperator::BitwiseAnd => OpCode::BitAnd,
            BinaryOperator::BitwiseOr => OpCode::BitOr,
            BinaryOperator::BitwiseXor => OpCode::BitXor,
            BinaryOperator::LeftShift => OpCode::Shl,
            BinaryOperator::RightShift => OpCode::Shr,
            BinaryOperator::UnsignedRightShift => OpCode::Ushr,
            BinaryOperator::In => OpCode::In,
            BinaryOperator::InstanceOf => OpCode::InstanceOf,
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => unreachable!(),
            BinaryOperator::NullishCoalescing => {
                // ES2020 feature, treat as logical or for now
                OpCode::BitOr
            }
        };

        self.emit(Instruction::simple(opcode));
        Ok(())
    }

    fn compile_unary(&mut self, un: &UnaryExpression) -> Result<(), Error> {
        match un.operator {
            UnaryOperator::Typeof => {
                self.compile_expression(&un.argument)?;
                self.emit(Instruction::simple(OpCode::TypeOf));
            }
            UnaryOperator::Delete => {
                if let Expression::Member(member) = un.argument.as_ref() {
                    self.compile_expression(&member.object)?;
                    match &member.property {
                        MemberProperty::Identifier(id) => {
                            let idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                            self.emit(Instruction::with_operand(
                                OpCode::LoadConst,
                                Operand::Constant(idx),
                            ));
                        }
                        MemberProperty::Expression(expr) => {
                            self.compile_expression(expr)?;
                        }
                    }
                    self.emit(Instruction::simple(OpCode::DeleteProperty));
                } else {
                    // delete on non-member always returns true in ES5
                    self.emit(Instruction::simple(OpCode::LoadTrue));
                }
            }
            UnaryOperator::Void => {
                self.compile_expression(&un.argument)?;
                self.emit(Instruction::simple(OpCode::Pop));
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }
            _ => {
                self.compile_expression(&un.argument)?;
                let opcode = match un.operator {
                    UnaryOperator::Minus => OpCode::Neg,
                    UnaryOperator::Plus => {
                        // Unary + converts to number (no-op for now)
                        return Ok(());
                    }
                    UnaryOperator::LogicalNot => OpCode::Not,
                    UnaryOperator::BitwiseNot => OpCode::BitNot,
                    _ => unreachable!(),
                };
                self.emit(Instruction::simple(opcode));
            }
        }
        Ok(())
    }

    fn compile_assignment(&mut self, assign: &AssignmentExpression) -> Result<(), Error> {
        // Compile the right-hand side
        match assign.operator {
            AssignmentOperator::Assign => {
                self.compile_expression(&assign.right)?;
            }
            _ => {
                // Compound assignment: load left, apply op, store
                self.compile_expression(&assign.left)?;
                self.compile_expression(&assign.right)?;

                let opcode = match assign.operator {
                    AssignmentOperator::AddAssign => OpCode::Add,
                    AssignmentOperator::SubtractAssign => OpCode::Sub,
                    AssignmentOperator::MultiplyAssign => OpCode::Mul,
                    AssignmentOperator::DivideAssign => OpCode::Div,
                    AssignmentOperator::ModuloAssign => OpCode::Mod,
                    AssignmentOperator::ExponentAssign => OpCode::Pow,
                    AssignmentOperator::LeftShiftAssign => OpCode::Shl,
                    AssignmentOperator::RightShiftAssign => OpCode::Shr,
                    AssignmentOperator::UnsignedRightShiftAssign => OpCode::Ushr,
                    AssignmentOperator::BitwiseAndAssign => OpCode::BitAnd,
                    AssignmentOperator::BitwiseOrAssign => OpCode::BitOr,
                    AssignmentOperator::BitwiseXorAssign => OpCode::BitXor,
                    _ => {
                        return Err(Error::InternalError(
                            "Unsupported assignment operator".into(),
                        ));
                    }
                };
                self.emit(Instruction::simple(opcode));
            }
        }

        // Store to target
        match assign.left.as_ref() {
            Expression::Identifier(id) => {
                // Duplicate value so it remains on stack
                self.emit(Instruction::simple(OpCode::Dup));

                if let Some(idx) = self.resolve_local(&id.name) {
                    self.emit(Instruction::with_operand(
                        OpCode::StoreLocal,
                        Operand::Local(idx),
                    ));
                } else {
                    let name_idx = self.add_global_name(&id.name);
                    self.emit(Instruction::with_operand(
                        OpCode::StoreGlobal,
                        Operand::Constant(name_idx),
                    ));
                }
            }
            Expression::Member(member) => {
                // Need to compile object and property, then set
                self.compile_expression(&member.object)?;
                match &member.property {
                    MemberProperty::Identifier(id) => {
                        let idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::LoadConst,
                            Operand::Constant(idx),
                        ));
                    }
                    MemberProperty::Expression(expr) => {
                        self.compile_expression(expr)?;
                    }
                }
                self.emit(Instruction::simple(OpCode::SetProperty));
            }
            _ => {
                return Err(Error::SyntaxError("Invalid assignment target".into()));
            }
        }

        Ok(())
    }

    fn compile_call(&mut self, call: &CallExpression) -> Result<(), Error> {
        self.compile_expression(&call.callee)?;

        for arg in &call.arguments {
            match arg {
                Argument::Expression(expr) => {
                    self.compile_expression(expr)?;
                }
                Argument::Spread(expr) => {
                    // For now, compile spread as regular expression
                    // Full spread support requires runtime iteration
                    self.compile_expression(expr)?;
                }
            }
        }

        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(call.arguments.len() as u8),
        ));
        Ok(())
    }

    fn compile_member(&mut self, member: &MemberExpression, _for_call: bool) -> Result<(), Error> {
        self.compile_expression(&member.object)?;

        match &member.property {
            MemberProperty::Identifier(id) => {
                let idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
            }
            MemberProperty::Expression(expr) => {
                self.compile_expression(expr)?;
            }
        }

        self.emit(Instruction::simple(OpCode::GetProperty));
        Ok(())
    }

    fn compile_conditional(&mut self, cond: &ConditionalExpression) -> Result<(), Error> {
        self.compile_expression(&cond.test)?;

        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit(Instruction::simple(OpCode::Pop));

        self.compile_expression(&cond.consequent)?;

        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit(Instruction::simple(OpCode::Pop));

        self.compile_expression(&cond.alternate)?;

        self.patch_jump(end_jump);

        Ok(())
    }

    fn compile_function_expression(&mut self, _func: &FunctionExpression) -> Result<(), Error> {
        // For now, push a placeholder
        // A real implementation would compile the function body and create a closure
        self.emit(Instruction::simple(OpCode::LoadUndefined));
        Ok(())
    }

    fn compile_arrow_function(&mut self, arrow: &ArrowFunctionExpression) -> Result<(), Error> {
        // Arrow functions capture lexical 'this' and don't have their own 'this'
        // For now, we'll compile them similarly to function expressions
        // A full implementation would:
        // 1. Create a new function object
        // 2. Capture the enclosing 'this' value
        // 3. Set up parameter bindings
        // 4. Compile the body

        // For now, push a placeholder - full function compilation will be implemented
        // when we add proper closure support
        let _ = arrow; // Suppress unused warning
        self.emit(Instruction::simple(OpCode::LoadUndefined));
        Ok(())
    }

    fn compile_template_literal(&mut self, template: &TemplateLiteral) -> Result<(), Error> {
        // Template literals are compiled as string concatenation
        // `Hello ${name}!` becomes: "Hello " + String(name) + "!"

        if template.expressions.is_empty() {
            // No substitutions, just a simple string
            let value = &template.quasis[0].cooked;
            let idx = self.bytecode.add_constant(Value::String(value.clone()));
            self.emit(Instruction::with_operand(
                OpCode::LoadConst,
                Operand::Constant(idx),
            ));
            return Ok(());
        }

        // Start with the first quasi
        let first_quasi = &template.quasis[0].cooked;
        let idx = self
            .bytecode
            .add_constant(Value::String(first_quasi.clone()));
        self.emit(Instruction::with_operand(
            OpCode::LoadConst,
            Operand::Constant(idx),
        ));

        // Interleave expressions and quasis
        for (i, expr) in template.expressions.iter().enumerate() {
            // Compile the expression
            self.compile_expression(expr)?;
            // Convert to string (implicit in Add for our VM)
            self.emit(Instruction::simple(OpCode::Add));

            // Add the next quasi
            let quasi = &template.quasis[i + 1].cooked;
            if !quasi.is_empty() {
                let idx = self.bytecode.add_constant(Value::String(quasi.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
                self.emit(Instruction::simple(OpCode::Add));
            }
        }

        Ok(())
    }

    fn compile_tagged_template(&mut self, tagged: &TaggedTemplateExpression) -> Result<(), Error> {
        // Tagged templates call the tag function with:
        // 1. An array of the static string parts (quasis)
        // 2. The interpolated values as additional arguments

        // Compile the tag function
        self.compile_expression(&tagged.tag)?;

        // Create the template strings array
        let quasis_count = tagged.quasi.quasis.len();
        for quasi in &tagged.quasi.quasis {
            let idx = self
                .bytecode
                .add_constant(Value::String(quasi.cooked.clone()));
            self.emit(Instruction::with_operand(
                OpCode::LoadConst,
                Operand::Constant(idx),
            ));
        }
        self.emit(Instruction::with_operand(
            OpCode::NewArray,
            Operand::ArgCount(quasis_count as u8),
        ));

        // Compile each expression as an argument
        for expr in &tagged.quasi.expressions {
            self.compile_expression(expr)?;
        }

        // Call the tag function
        let arg_count = 1 + tagged.quasi.expressions.len(); // strings array + expressions
        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(arg_count as u8),
        ));

        Ok(())
    }

    fn compile_new(&mut self, new_expr: &NewExpression) -> Result<(), Error> {
        // Compile constructor
        self.compile_expression(&new_expr.callee)?;

        // Compile arguments
        for arg in &new_expr.arguments {
            match arg {
                Argument::Expression(expr) => {
                    self.compile_expression(expr)?;
                }
                Argument::Spread(expr) => {
                    // For now, compile spread as regular expression
                    self.compile_expression(expr)?;
                }
            }
        }

        // New expression (uses Call with a flag, but we'll use Call for now)
        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(new_expr.arguments.len() as u8),
        ));

        Ok(())
    }

    fn compile_update(&mut self, update: &UpdateExpression) -> Result<(), Error> {
        let op = match update.operator {
            UpdateOperator::Increment => OpCode::Add,
            UpdateOperator::Decrement => OpCode::Sub,
        };

        match update.argument.as_ref() {
            Expression::Identifier(id) => {
                if update.prefix {
                    // ++x: load, add 1, store, return new value
                    self.compile_identifier(id)?;
                    let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                    self.emit(Instruction::with_operand(
                        OpCode::LoadConst,
                        Operand::Constant(one_idx),
                    ));
                    self.emit(Instruction::simple(op));
                    self.emit(Instruction::simple(OpCode::Dup));

                    if let Some(idx) = self.resolve_local(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(idx),
                        ));
                    } else {
                        let name_idx = self.add_global_name(&id.name);
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Constant(name_idx),
                        ));
                    }
                } else {
                    // x++: load, dup, add 1, store, return old value
                    self.compile_identifier(id)?;
                    self.emit(Instruction::simple(OpCode::Dup));
                    let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                    self.emit(Instruction::with_operand(
                        OpCode::LoadConst,
                        Operand::Constant(one_idx),
                    ));
                    self.emit(Instruction::simple(op));

                    if let Some(idx) = self.resolve_local(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(idx),
                        ));
                    } else {
                        let name_idx = self.add_global_name(&id.name);
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Constant(name_idx),
                        ));
                    }
                }
            }
            _ => {
                return Err(Error::SyntaxError(
                    "Invalid update expression operand".into(),
                ));
            }
        }

        Ok(())
    }

    fn compile_sequence(&mut self, seq: &SequenceExpression) -> Result<(), Error> {
        for (i, expr) in seq.expressions.iter().enumerate() {
            self.compile_expression(expr)?;
            if i < seq.expressions.len() - 1 {
                self.emit(Instruction::simple(OpCode::Pop));
            }
        }
        Ok(())
    }

    // ==================== Helper Methods ====================

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.bytecode.emit(instruction)
    }

    fn emit_jump(&mut self, opcode: OpCode) -> usize {
        self.emit(Instruction::with_operand(opcode, Operand::Jump(0)))
    }

    fn patch_jump(&mut self, jump_idx: usize) {
        let offset = self.bytecode.instructions.len() as i32 - jump_idx as i32 - 1;
        if let Some(Operand::Jump(_)) = self.bytecode.instructions[jump_idx].operand {
            self.bytecode.instructions[jump_idx].operand = Some(Operand::Jump(offset));
        }
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = loop_start as i32 - self.bytecode.instructions.len() as i32 - 1;
        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(offset),
        ));
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Pop locals that are going out of scope
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.locals.pop();
            self.emit(Instruction::simple(OpCode::Pop));
        }
    }

    fn add_local(&mut self, name: &str, kind: LocalKind) {
        self.locals.push(Local {
            name: name.to_string(),
            depth: self.scope_depth,
            kind,
            initialized: kind == LocalKind::Var || kind == LocalKind::Parameter, // var is hoisted
        });
    }

    #[allow(dead_code)]
    fn mark_local_initialized(&mut self, name: &str) {
        for local in self.locals.iter_mut().rev() {
            if local.name == name {
                local.initialized = true;
                return;
            }
        }
    }

    fn resolve_local(&self, name: &str) -> Option<u16> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i as u16);
            }
        }
        None
    }

    fn check_local_tdz(&self, name: &str) -> Result<(), Error> {
        for local in self.locals.iter().rev() {
            if local.name == name {
                if !local.initialized
                    && (local.kind == LocalKind::Let || local.kind == LocalKind::Const)
                {
                    return Err(Error::ReferenceError(format!(
                        "Cannot access '{}' before initialization",
                        name
                    )));
                }
                return Ok(());
            }
        }
        Ok(())
    }

    fn add_global_name(&mut self, name: &str) -> u16 {
        if let Some(&idx) = self.globals.get(name) {
            idx
        } else {
            let idx = self.bytecode.add_constant(Value::String(name.to_string()));
            self.globals.insert(name.to_string(), idx);
            idx
        }
    }

    fn push_loop(&mut self, label: Option<String>) {
        self.loop_stack.push(LoopContext {
            label,
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
            continue_target: None,
        });
    }

    fn set_continue_target(&mut self, target: usize) {
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.continue_target = Some(target);
        }
    }

    fn pop_loop(&mut self) {
        if let Some(ctx) = self.loop_stack.pop() {
            // Patch all break jumps to current position
            let current = self.bytecode.instructions.len();
            for jump_idx in ctx.break_jumps {
                let offset = current as i32 - jump_idx as i32 - 1;
                self.bytecode.instructions[jump_idx].operand = Some(Operand::Jump(offset));
            }
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn compile_str(source: &str) -> Result<Bytecode, Error> {
        let mut parser = Parser::new(source);
        let program = parser.parse_program()?;
        let mut compiler = Compiler::new();
        compiler.compile(&program)
    }

    #[test]
    fn test_compile_literals() {
        let bytecode = compile_str("42;").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_variable_declaration() {
        let bytecode = compile_str("var x = 10;").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_if_statement() {
        let bytecode = compile_str("if (true) { 1; } else { 2; }").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_while_loop() {
        let bytecode = compile_str("while (x) { x--; }").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_for_loop() {
        let bytecode = compile_str("for (var i = 0; i < 10; i++) { i; }").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_function_call() {
        let bytecode = compile_str("foo(1, 2, 3);").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_object_literal() {
        let bytecode = compile_str("var obj = { a: 1, b: 2 };").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_array_literal() {
        let bytecode = compile_str("var arr = [1, 2, 3];").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_conditional_expression() {
        let bytecode = compile_str("x ? 1 : 2;").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_short_circuit() {
        let bytecode = compile_str("a && b || c;").unwrap();
        assert!(!bytecode.instructions.is_empty());
    }
}
