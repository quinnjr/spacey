//! Abstract Syntax Tree (AST) definitions for JavaScript.
//!
//! These structures are designed to be ESTree-compatible where possible.

/// A complete JavaScript program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The statements in the program
    pub body: Vec<Statement>,
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
    /// Expression statement
    Expression(ExpressionStatement),
    /// Block statement { ... }
    Block(BlockStatement),
    /// If statement
    If(IfStatement),
    /// Switch statement
    Switch(SwitchStatement),
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
    /// Return statement
    Return(ReturnStatement),
    /// Break statement (with optional label)
    Break,
    /// Break with label
    BreakLabel(String),
    /// Continue statement
    Continue,
    /// Continue with label
    ContinueLabel(String),
    /// Throw statement
    Throw(ThrowStatement),
    /// Try statement
    Try(TryStatement),
    /// With statement (ES3, deprecated in strict mode)
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
    /// The identifier being declared
    pub id: Identifier,
    /// Optional initializer expression
    pub init: Option<Expression>,
}

/// A function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The function name
    pub id: Identifier,
    /// The parameters
    pub params: Vec<Identifier>,
    /// The function body
    pub body: Vec<Statement>,
    /// Whether this is an async function
    pub is_async: bool,
    /// Whether this is a generator function
    pub is_generator: bool,
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

/// A switch statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStatement {
    /// The discriminant expression
    pub discriminant: Expression,
    /// The case clauses
    pub cases: Vec<SwitchCase>,
}

/// A switch case clause.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// The test expression (None for default)
    pub test: Option<Expression>,
    /// The consequent statements
    pub consequent: Vec<Statement>,
}

/// A do-while statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStatement {
    /// The loop body
    pub body: Box<Statement>,
    /// The condition
    pub test: Expression,
}

/// A for-in statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ForInStatement {
    /// The left-hand side
    pub left: ForInLeft,
    /// The object to iterate over
    pub right: Expression,
    /// The loop body
    pub body: Box<Statement>,
}

/// A for-of statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ForOfStatement {
    /// The left-hand side
    pub left: ForInLeft,
    /// The iterable
    pub right: Expression,
    /// The loop body
    pub body: Box<Statement>,
    /// Whether this is an async for-of
    pub is_await: bool,
}

/// Left-hand side of for-in/for-of.
#[derive(Debug, Clone, PartialEq)]
pub enum ForInLeft {
    /// Variable declaration
    Declaration(Box<VariableDeclaration>),
    /// Expression (identifier or member)
    Expression(Expression),
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

/// A with statement (ES3 Section 12.10).
/// Note: Deprecated in ES5 strict mode but required for ES3 compatibility.
#[derive(Debug, Clone, PartialEq)]
pub struct WithStatement {
    /// The object expression
    pub object: Expression,
    /// The body statement
    pub body: Box<Statement>,
}

/// A labeled statement (ES3 Section 12.12).
#[derive(Debug, Clone, PartialEq)]
pub struct LabeledStatement {
    /// The label identifier
    pub label: Identifier,
    /// The labeled body
    pub body: Box<Statement>,
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
    #[allow(missing_docs)]
    RegExp { pattern: String, flags: String },
}

/// An array expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpression {
    /// The elements (None represents a hole)
    pub elements: Vec<Option<Expression>>,
}

/// An object expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectExpression {
    /// The properties
    pub properties: Vec<Property>,
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
#[allow(missing_docs)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    // Comparison
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    // Logical
    LogicalAnd,
    LogicalOr,
    NullishCoalescing,
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    // Other
    In,
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
#[allow(missing_docs)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    ExponentAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LogicalAndAssign,
    LogicalOrAssign,
    NullishCoalescingAssign,
}

/// A function call expression.
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    /// The function being called
    pub callee: Box<Expression>,
    /// The arguments
    pub arguments: Vec<Expression>,
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
    pub params: Vec<Identifier>,
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
    pub params: Vec<Identifier>,
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
    /// The arguments
    pub arguments: Vec<Expression>,
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
