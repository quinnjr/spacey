//! Abstract Syntax Tree (AST) definitions for JavaScript.
//!
//! These structures are designed to be ESTree-compatible where possible.

/// A complete JavaScript program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The statements and module items in the program
    pub body: Vec<ModuleItem>,
    /// Whether this is a module (has import/export)
    pub source_type: SourceType,
}

/// The type of source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceType {
    /// Script (no imports/exports)
    #[default]
    Script,
    /// ES Module
    Module,
}

/// A module item (either a statement or a module declaration).
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleItem {
    /// A regular statement
    Statement(Statement),
    /// An import declaration
    ImportDeclaration(ImportDeclaration),
    /// An export declaration
    ExportDeclaration(ExportDeclaration),
}

/// An import declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDeclaration {
    /// Import specifiers
    pub specifiers: Vec<ImportSpecifier>,
    /// The module source
    pub source: String,
}

/// An import specifier.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportSpecifier {
    /// import foo from 'module'
    Default(Identifier),
    /// import * as foo from 'module'
    Namespace(Identifier),
    /// import { foo } from 'module' or import { foo as bar } from 'module'
    Named {
        /// The imported name
        imported: Identifier,
        /// The local binding name
        local: Identifier,
    },
}

/// An export declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportDeclaration {
    /// export { foo, bar }
    Named {
        /// Export specifiers
        specifiers: Vec<ExportSpecifier>,
        /// Optional source module for re-exports
        source: Option<String>,
    },
    /// export default expression
    Default(Box<Expression>),
    /// export const/let/var/function/class declaration
    Declaration(Box<Statement>),
    /// export * from 'module'
    All {
        /// The module source
        source: String,
        /// Optional namespace export name (export * as foo from 'module')
        exported: Option<Identifier>,
    },
}

/// An export specifier.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportSpecifier {
    /// The local name
    pub local: Identifier,
    /// The exported name (may differ from local)
    pub exported: Identifier,
}

/// An identifier.
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    /// The name of the identifier
    pub name: String,
}

/// A JavaScript statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable declaration (var, let, const)
    VariableDeclaration(VariableDeclaration),
    /// Function declaration
    FunctionDeclaration(FunctionDeclaration),
    /// Class declaration
    ClassDeclaration(ClassDeclaration),
    /// Expression statement
    Expression(ExpressionStatement),
    /// Block statement { ... }
    Block(BlockStatement),
    /// If statement
    If(IfStatement),
    /// While statement
    While(WhileStatement),
    /// Do-while statement
    DoWhile(DoWhileStatement),
    /// For statement
    For(ForStatement),
    /// For-in statement
    ForIn(ForInStatement),
    /// For-of statement
    ForOf(ForOfStatement),
    /// Switch statement
    Switch(SwitchStatement),
    /// Return statement
    Return(ReturnStatement),
    /// Break statement
    Break(BreakStatement),
    /// Continue statement
    Continue(ContinueStatement),
    /// Throw statement
    Throw(ThrowStatement),
    /// Try statement
    Try(TryStatement),
    /// With statement
    With(WithStatement),
    /// Labeled statement
    Labeled(LabeledStatement),
    /// Debugger statement
    Debugger,
    /// Empty statement (;)
    Empty,
}

/// Variable declaration kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKind {
    /// var declaration
    Var,
    /// let declaration
    Let,
    /// const declaration
    Const,
}

/// A variable declaration statement.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    /// The kind of declaration
    pub kind: VariableKind,
    /// The declarators
    pub declarations: Vec<VariableDeclarator>,
}

/// A single variable declarator.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarator {
    /// The binding pattern (identifier or destructuring)
    pub id: BindingPattern,
    /// Optional initializer expression
    pub init: Option<Expression>,
}

/// A function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The function name
    pub id: Identifier,
    /// The parameters
    pub params: Vec<Parameter>,
    /// The function body
    pub body: Vec<Statement>,
    /// Whether this is an async function
    pub is_async: bool,
    /// Whether this is a generator function
    pub is_generator: bool,
}

/// A function parameter with optional default value.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// The parameter binding pattern (identifier or destructuring)
    pub pattern: BindingPattern,
    /// Optional default value
    pub default: Option<Expression>,
}

/// A binding pattern for variable declarations and parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum BindingPattern {
    /// Simple identifier binding
    Identifier(Identifier),
    /// Array destructuring pattern
    Array(ArrayBindingPattern),
    /// Object destructuring pattern
    Object(ObjectBindingPattern),
    /// Rest element (...args)
    Rest(Box<BindingPattern>),
}

/// Array destructuring pattern: [a, b, ...rest]
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayBindingPattern {
    /// Elements (None represents holes)
    pub elements: Vec<Option<ArrayBindingElement>>,
    /// Optional rest element
    pub rest: Option<Box<BindingPattern>>,
}

/// An element in an array binding pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayBindingElement {
    /// The binding pattern
    pub pattern: BindingPattern,
    /// Optional default value
    pub default: Option<Expression>,
}

/// Object destructuring pattern: {a, b: c, ...rest}
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectBindingPattern {
    /// Properties to bind
    pub properties: Vec<BindingProperty>,
    /// Optional rest element
    pub rest: Option<Identifier>,
}

/// A property in an object binding pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingProperty {
    /// The key (property name)
    pub key: PropertyKey,
    /// The value pattern (for renamed bindings)
    pub value: BindingPattern,
    /// Whether this is shorthand: {a} instead of {a: a}
    pub shorthand: bool,
    /// Optional default value
    pub default: Option<Expression>,
}

/// An expression statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStatement {
    /// The expression
    pub expression: Expression,
}

/// A block statement.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
    /// The statements in the block
    pub body: Vec<Statement>,
}

/// An if statement.
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    /// The condition
    pub test: Expression,
    /// The then branch
    pub consequent: Box<Statement>,
    /// The optional else branch
    pub alternate: Option<Box<Statement>>,
}

/// A while statement.
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    /// The condition
    pub test: Expression,
    /// The loop body
    pub body: Box<Statement>,
}

/// A for statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    /// The initializer
    pub init: Option<ForInit>,
    /// The condition
    pub test: Option<Expression>,
    /// The update expression
    pub update: Option<Expression>,
    /// The loop body
    pub body: Box<Statement>,
}

/// For loop initializer.
#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    /// Variable declaration
    Declaration(Box<VariableDeclaration>),
    /// Expression
    Expression(Expression),
}

/// A do-while statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStatement {
    /// The condition
    pub test: Expression,
    /// The loop body
    pub body: Box<Statement>,
}

/// A for-in statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ForInStatement {
    /// The left-hand side (variable declaration or expression)
    pub left: ForInLeft,
    /// The object to iterate over
    pub right: Expression,
    /// The loop body
    pub body: Box<Statement>,
}

/// Left-hand side of a for-in loop.
#[derive(Debug, Clone, PartialEq)]
pub enum ForInLeft {
    /// Variable declaration
    Declaration(Box<VariableDeclaration>),
    /// Expression (identifier or pattern)
    Expression(Expression),
}

/// A for-of statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ForOfStatement {
    /// The left-hand side (variable declaration or pattern)
    pub left: ForInLeft,
    /// The iterable to iterate over
    pub right: Expression,
    /// The loop body
    pub body: Box<Statement>,
    /// Whether this is a for-await-of loop
    pub is_await: bool,
}

/// A switch statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStatement {
    /// The discriminant expression
    pub discriminant: Expression,
    /// The switch cases
    pub cases: Vec<SwitchCase>,
}

/// A switch case.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// The test expression (None for default case)
    pub test: Option<Expression>,
    /// The consequent statements
    pub consequent: Vec<Statement>,
}

/// A break statement.
#[derive(Debug, Clone, PartialEq)]
pub struct BreakStatement {
    /// Optional label
    pub label: Option<Identifier>,
}

/// A continue statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStatement {
    /// Optional label
    pub label: Option<Identifier>,
}

/// A with statement.
#[derive(Debug, Clone, PartialEq)]
pub struct WithStatement {
    /// The object expression
    pub object: Expression,
    /// The body statement
    pub body: Box<Statement>,
}

/// A labeled statement.
#[derive(Debug, Clone, PartialEq)]
pub struct LabeledStatement {
    /// The label identifier
    pub label: Identifier,
    /// The body statement
    pub body: Box<Statement>,
}

/// A return statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    /// The return value
    pub argument: Option<Expression>,
}

/// A throw statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ThrowStatement {
    /// The thrown expression
    pub argument: Expression,
}

/// A try statement.
#[derive(Debug, Clone, PartialEq)]
pub struct TryStatement {
    /// The try block
    pub block: BlockStatement,
    /// The catch clause
    pub handler: Option<CatchClause>,
    /// The finally block
    pub finalizer: Option<BlockStatement>,
}

/// A catch clause.
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    /// The error parameter
    pub param: Option<Identifier>,
    /// The catch body
    pub body: BlockStatement,
}

/// A JavaScript expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal value
    Literal(Literal),
    /// Identifier reference
    Identifier(Identifier),
    /// this keyword
    This,
    /// Array literal
    Array(ArrayExpression),
    /// Object literal
    Object(ObjectExpression),
    /// Binary expression
    Binary(BinaryExpression),
    /// Unary expression
    Unary(UnaryExpression),
    /// Assignment expression
    Assignment(AssignmentExpression),
    /// Call expression
    Call(CallExpression),
    /// Member access expression
    Member(MemberExpression),
    /// Conditional (ternary) expression
    Conditional(ConditionalExpression),
    /// Function expression
    Function(FunctionExpression),
    /// Arrow function expression
    Arrow(ArrowFunctionExpression),
    /// new expression
    New(NewExpression),
    /// Update expression (++/--)
    Update(UpdateExpression),
    /// Sequence expression (comma operator)
    Sequence(SequenceExpression),
    /// Template literal
    TemplateLiteral(TemplateLiteral),
    /// Tagged template expression
    TaggedTemplate(TaggedTemplateExpression),
    /// Class expression
    Class(ClassExpression),
    /// super keyword
    Super,
    /// yield expression (in generators)
    Yield(YieldExpression),
    /// await expression (in async functions)
    Await(AwaitExpression),
}

/// A yield expression in a generator function.
#[derive(Debug, Clone, PartialEq)]
pub struct YieldExpression {
    /// The value to yield
    pub argument: Option<Box<Expression>>,
    /// Whether this is yield* (delegating)
    pub delegate: bool,
}

/// An await expression in an async function.
#[derive(Debug, Clone, PartialEq)]
pub struct AwaitExpression {
    /// The value to await
    pub argument: Box<Expression>,
}

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Numeric literal
    Number(f64),
    /// String literal
    String(String),
    /// Boolean literal
    Boolean(bool),
    /// null literal
    Null,
    /// undefined literal
    Undefined,
    /// BigInt literal
    BigInt(String),
    /// Regular expression literal
    RegExp {
        /// The regex pattern
        pattern: String,
        /// The regex flags
        flags: String,
    },
}

/// An array expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpression {
    /// The elements (None represents a hole)
    pub elements: Vec<Option<ArrayElement>>,
}

/// An element in an array literal.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElement {
    /// Regular expression element
    Expression(Expression),
    /// Spread element: ...expr
    Spread(Expression),
}

/// An object expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectExpression {
    /// The properties (regular properties or spread)
    pub properties: Vec<ObjectProperty>,
}

/// An object property or spread element.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectProperty {
    /// Regular property
    Property(Property),
    /// Spread element: ...expr
    Spread(Expression),
}

/// An object property.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// The property key
    pub key: PropertyKey,
    /// The property value
    pub value: Expression,
    /// Whether this is shorthand syntax
    pub shorthand: bool,
}

/// A property key.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKey {
    /// Identifier key
    Identifier(Identifier),
    /// Computed key
    Computed(Box<Expression>),
    /// Literal key (e.g., numeric or string)
    Literal(Literal),
}

/// A binary expression.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpression {
    /// The operator
    pub operator: BinaryOperator,
    /// The left operand
    pub left: Box<Expression>,
    /// The right operand
    pub right: Box<Expression>,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Subtract,
    /// Multiplication (*)
    Multiply,
    /// Division (/)
    Divide,
    /// Modulo (%)
    Modulo,
    /// Exponentiation (**)
    Exponent,
    // Comparison
    /// Equality (==)
    Equal,
    /// Inequality (!=)
    NotEqual,
    /// Strict equality (===)
    StrictEqual,
    /// Strict inequality (!==)
    StrictNotEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanEqual,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanEqual,
    // Logical
    /// Logical AND (&&)
    LogicalAnd,
    /// Logical OR (||)
    LogicalOr,
    /// Nullish coalescing (??)
    NullishCoalescing,
    // Bitwise
    /// Bitwise AND (&)
    BitwiseAnd,
    /// Bitwise OR (|)
    BitwiseOr,
    /// Bitwise XOR (^)
    BitwiseXor,
    /// Left shift (<<)
    LeftShift,
    /// Right shift (>>)
    RightShift,
    /// Unsigned right shift (>>>)
    UnsignedRightShift,
    // Other
    /// in operator
    In,
    /// instanceof operator
    InstanceOf,
}

/// A unary expression.
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpression {
    /// The operator
    pub operator: UnaryOperator,
    /// The operand
    pub argument: Box<Expression>,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// -
    Minus,
    /// +
    Plus,
    /// !
    LogicalNot,
    /// ~
    BitwiseNot,
    /// typeof
    Typeof,
    /// void
    Void,
    /// delete
    Delete,
}

/// An assignment expression.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpression {
    /// The operator
    pub operator: AssignmentOperator,
    /// The left-hand side
    pub left: Box<Expression>,
    /// The right-hand side
    pub right: Box<Expression>,
}

/// Assignment operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    /// Simple assignment (=)
    Assign,
    /// Addition assignment (+=)
    AddAssign,
    /// Subtraction assignment (-=)
    SubtractAssign,
    /// Multiplication assignment (*=)
    MultiplyAssign,
    /// Division assignment (/=)
    DivideAssign,
    /// Modulo assignment (%=)
    ModuloAssign,
    /// Exponentiation assignment (**=)
    ExponentAssign,
    /// Left shift assignment (<<=)
    LeftShiftAssign,
    /// Right shift assignment (>>=)
    RightShiftAssign,
    /// Unsigned right shift assignment (>>>=)
    UnsignedRightShiftAssign,
    /// Bitwise AND assignment (&=)
    BitwiseAndAssign,
    /// Bitwise OR assignment (|=)
    BitwiseOrAssign,
    /// Bitwise XOR assignment (^=)
    BitwiseXorAssign,
    /// Logical AND assignment (&&=)
    LogicalAndAssign,
    /// Logical OR assignment (||=)
    LogicalOrAssign,
    /// Nullish coalescing assignment (??=)
    NullishCoalescingAssign,
}

/// A function call expression.
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    /// The function being called
    pub callee: Box<Expression>,
    /// The arguments (may include spread)
    pub arguments: Vec<Argument>,
    /// Whether this is optional chaining (?.)
    pub optional: bool,
}

/// An argument in a function call.
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// Regular expression argument
    Expression(Expression),
    /// Spread argument: ...expr
    Spread(Expression),
}

/// A member access expression.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpression {
    /// The object
    pub object: Box<Expression>,
    /// The property
    pub property: MemberProperty,
    /// Whether this is computed (bracket notation)
    pub computed: bool,
    /// Whether this is optional chaining (?.)
    pub optional: bool,
}

/// Member property.
#[derive(Debug, Clone, PartialEq)]
pub enum MemberProperty {
    /// Identifier property
    Identifier(Identifier),
    /// Computed property expression
    Expression(Box<Expression>),
}

/// A conditional (ternary) expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpression {
    /// The condition
    pub test: Box<Expression>,
    /// The consequent (if true)
    pub consequent: Box<Expression>,
    /// The alternate (if false)
    pub alternate: Box<Expression>,
}

/// A function expression.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionExpression {
    /// Optional name
    pub id: Option<Identifier>,
    /// Parameters
    pub params: Vec<Parameter>,
    /// Body
    pub body: Vec<Statement>,
    /// Whether async
    pub is_async: bool,
    /// Whether generator
    pub is_generator: bool,
}

/// An arrow function expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrowFunctionExpression {
    /// Parameters
    pub params: Vec<Parameter>,
    /// Body (expression or block)
    pub body: ArrowBody,
    /// Whether async
    pub is_async: bool,
}

/// Arrow function body.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody {
    /// Expression body
    Expression(Box<Expression>),
    /// Block body
    Block(Vec<Statement>),
}

/// A new expression.
#[derive(Debug, Clone, PartialEq)]
pub struct NewExpression {
    /// The constructor
    pub callee: Box<Expression>,
    /// The arguments (may include spread)
    pub arguments: Vec<Argument>,
}

/// An update expression (++/--)
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateExpression {
    /// The operator
    pub operator: UpdateOperator,
    /// The operand
    pub argument: Box<Expression>,
    /// Whether prefix (++x) or postfix (x++)
    pub prefix: bool,
}

/// Update operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOperator {
    /// ++
    Increment,
    /// --
    Decrement,
}

/// A sequence expression (comma operator).
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceExpression {
    /// The expressions
    pub expressions: Vec<Expression>,
}

/// A template literal element (the string parts).
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateElement {
    /// The cooked value (with escape sequences processed)
    pub cooked: String,
    /// The raw value (as written in source)
    pub raw: String,
    /// Whether this is the last element
    pub tail: bool,
}

/// A template literal expression: `Hello ${name}!`
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateLiteral {
    /// The static string parts (quasis)
    pub quasis: Vec<TemplateElement>,
    /// The interpolated expressions (one fewer than quasis)
    pub expressions: Vec<Expression>,
}

/// A tagged template expression: tag`string`
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedTemplateExpression {
    /// The tag function
    pub tag: Box<Expression>,
    /// The template literal
    pub quasi: TemplateLiteral,
}

/// A class declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclaration {
    /// The class name
    pub id: Identifier,
    /// The superclass (if any)
    pub super_class: Option<Box<Expression>>,
    /// The class body
    pub body: ClassBody,
}

/// A class expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassExpression {
    /// Optional class name
    pub id: Option<Identifier>,
    /// The superclass (if any)
    pub super_class: Option<Box<Expression>>,
    /// The class body
    pub body: ClassBody,
}

/// The body of a class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassBody {
    /// The class elements (methods, fields, static blocks)
    pub body: Vec<ClassElement>,
}

/// A class element (method, field, or static block).
#[derive(Debug, Clone, PartialEq)]
pub enum ClassElement {
    /// Method definition
    MethodDefinition(MethodDefinition),
    /// Property definition (class field)
    PropertyDefinition(PropertyDefinition),
    /// Static initialization block
    StaticBlock(StaticBlock),
}

/// A method definition in a class.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDefinition {
    /// Method key
    pub key: PropertyKey,
    /// Method value (function expression)
    pub value: FunctionExpression,
    /// Method kind
    pub kind: MethodKind,
    /// Whether this method is static
    pub is_static: bool,
    /// Whether this is a computed key
    pub computed: bool,
}

/// The kind of method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodKind {
    /// Regular method
    Method,
    /// Constructor
    Constructor,
    /// Getter
    Get,
    /// Setter
    Set,
}

/// A property definition (class field).
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDefinition {
    /// Property key
    pub key: PropertyKey,
    /// Optional initial value
    pub value: Option<Expression>,
    /// Whether this property is static
    pub is_static: bool,
    /// Whether this is a computed key
    pub computed: bool,
}

/// A static initialization block.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticBlock {
    /// The statements in the static block
    pub body: Vec<Statement>,
}
