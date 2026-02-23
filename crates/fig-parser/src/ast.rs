//! Abstract Syntax Tree definitions for Fig

use fig_lexer::{FloatLiteral, IntegerLiteral};
use serde::Serialize;

// ============================================================================
// Common / Shared Structures
// ============================================================================

/// A qualified path of identifiers, e.g. `std::Vec` or `Vec[T]`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Path {
    /// Segments of the path, e.g. `["std", "Vec"]`
    pub segments: Vec<String>,
}

impl Path {
    /// Convenience constructor for a single-segment path, e.g. `Path::simple("Vec")`.
    pub fn simple(name: String) -> Self {
        Path { segments: vec![name] }
    }
}

/// A single path segment in a function receiver or final function name, optionally with generic
/// parameters. e.g. `Vec[T]` in `func Vec[T]::push(...)`, or `add[U]` in `func Vec[T]::add[U](...)`.
/// Generic args here are always `GenericParameter` (simple identifiers or bounded identifiers)
/// since this position is always a declaration context.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GenericPathSegment {
    pub name: String,
    /// Generic parameters declared at this segment position.
    pub generic_args: Vec<GenericParameter>,
}

/// Visibility modifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub enum Visibility {
    /// No modifier. Private to the current namespace and its sub-namespaces.
    #[default]
    Default,

    /// Visible in the current package and all sub-packages, but not outside the package.
    Public,

    /// Visible outside the package, e.g. to other packages or when linking as a library.
    Export,

    /// Visible only to the type itself and its methods, e.g. for struct fields or enum variants.
    Private,
}

/// A single annotation, e.g. `#inline` or `#cfg(feature = "foo")`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<Expression>,
}

/// Self parameter in a method definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelfParameter {
    pub is_pointer: bool,
    pub is_mutable: bool,
}

// ============================================================================
// Expressions
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expression {
    // ── Literals ──
    IntegerLiteral(IntegerLiteral),
    FloatLiteral(FloatLiteral),
    BooleanLiteral(bool),
    CharLiteral(String),
    StringLiteral(String),
    OkLiteral,
    NullLiteral,

    // ── Name expressions ──
    /// The `self` keyword used as a value
    SelfValue,
    /// A (possibly qualified) path expression, e.g. `x`, `std::Vec`, `Vec[T]`
    Path(Path),

    // ── Composite literals ──
    ArrayLiteral(ArrayLiteralExpr),
    /// `[value; count]` — a fixed-size array filled with repeated copies of `value`
    RepeatArrayLiteral(RepeatArrayExpr),
    InterpolatedString(Vec<InterpolatedPart>),

    // ── Arithmetic / logical / bitwise ──
    BinaryOp(BinaryOpExpr),
    UnaryOp(UnaryOpExpr),

    // ── Postfix operations ──
    FieldAccess(FieldAccessExpr),
    TypeAccess(TypeAccessExpr),
    Call(CallExpr),
    Index(IndexExpr),

    // ── Casts and intrinsics ──
    Cast(CastExpr),
    Sizeof(Box<Type>),
    Alignof(Box<Type>),
    Offsetof(OffsetofExpr),

    // ── Grouping ──
    Parenthesized(Box<Expression>),

    // ── Assignment ──
    Assign(AssignExpr),
}

// ============================================================================
// Array / Interpolated String Literals
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayLiteralExpr {
    pub elements: Vec<Expression>,
}

/// `[value; count]` — fills a fixed-size array with `count` copies of `value`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RepeatArrayExpr {
    pub value: Box<Expression>,
    pub count: Box<Expression>,
}

/// A segment of an interpolated string
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum InterpolatedPart {
    Text(String),
    Expression(Box<Expression>),
}

// ============================================================================
// Postfix Operation Nodes
// ============================================================================

/// `object.field` or `object.!field`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FieldAccessExpr {
    pub object: Box<Expression>,
    pub field: String,
    pub is_propagating: bool,
}

/// `object::member`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypeAccessExpr {
    pub object: Box<Expression>,
    pub member: String,
}

/// A single argument in a function or constructor call.
/// Named form: `name: value` (used for struct construction and keyword args).
/// Positional form: `value`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CallArgument {
    pub name: Option<String>,
    pub value: Expression,
}

/// `callee(args)` or `callee!(args)`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CallExpr {
    pub callee: Box<Expression>,
    pub args: Vec<CallArgument>,
    pub generic_args: Option<Vec<Type>>,
    pub is_propagating: bool,
}

/// `object[index]`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndexExpr {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
}

/// `expr as Type`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CastExpr {
    pub expr: Box<Expression>,
    pub target_type: Box<Type>,
}

/// `offsetof(Type, field)`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OffsetofExpr {
    pub ty: Box<Type>,
    pub field: String,
}

// ============================================================================
// Assignment Operations
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AssignExpr {
    pub lhs: Box<Expression>,
    pub op: AssignOperator,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AssignOperator {
    Assign,       // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
}

// ============================================================================
// Binary Operations
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BinaryOpExpr {
    pub lhs: Box<Expression>,
    pub op: BinaryOperator,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
}

// ============================================================================
// Unary Operations
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnaryOpExpr {
    pub op: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum UnaryOperator {
    LogicalNot,  // !
    BitwiseNot,  // ~
    Negate,      // -
    Plus,        // +
    AddressOf,   // &
    Dereference, // *
}

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Type {
    // Primitive types
    U8,
    U16,
    U32,
    U64,
    USize,
    I8,
    I16,
    I32,
    I64,
    ISize,
    F32,
    F64,
    Bool,
    /// The `ok` type (successful/unit result)
    Ok,
    /// The `null` type
    Null,

    /// `Self` keyword as a type
    SelfType,

    /// Pointer type: `?*mut T`
    Pointer {
        nullable: bool,
        mutable: bool,
        element_type: Box<Type>,
    },

    /// Named / path type, e.g. `Vec[T]`, `std::HashMap[K, V]`
    Path { path: Path, generic_args: Vec<Type> },

    /// Array `[T; N]` or slice `[T]`
    Array {
        element_type: Box<Type>,
        /// Size expression for fixed arrays; `None` for slices
        size: Option<Box<Expression>>,
    },

    /// Nullable type: `?T` — the value is either `T` or `null`.
    Nullable(Box<Type>),

    /// Error-union type `T ! E` — the value is either `T` (ok) or an error of type `E`.
    /// Precedence: `*T ! E` = `(*T) ! E`, `?T ! E` = `(?T) ! E`.
    ErrorUnion {
        /// The success type (left-hand side of `!`)
        ok_type: Box<Type>,
        /// The error type (right-hand side of `!`), always a named path
        err_type: Box<Type>,
    },

    /// Function type: `func(T1, T2) -> R` or `fn(T1) -> R`.
    /// Both `func` and `fn` keywords produce this same node.
    /// Used in pointer-to-function positions, e.g. `*func(u8, u8) -> bool`.
    Function {
        /// Types of the parameters (positional only; no names in a function type).
        param_types: Vec<Type>,
        /// Return type.  `None` means the function type has no declared return
        /// (the language may implicitly treat it as `ok`).
        return_type: Option<Box<Type>>,
    },
}

// ============================================================================
// Generic Parameters
// ============================================================================

/// A single generic parameter or where-clause constraint
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GenericParameter {
    /// Type parameter: `T`, `T: Bound`, `T = Default`, `T: Bound = Default`
    Type {
        name: String,
        bounds: Vec<Type>,
        default_type: Option<Box<Type>>,
    },
    /// Const generic: `const N: usize`
    Const { name: String, ty: Type },
}

// ============================================================================
// Type Alias
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub name: Path,
    /// Combined generic params (bounds merged from param list + where clause)
    pub generic_params: Vec<GenericParameter>,
    pub aliased_type: Type,
}

// ============================================================================
// Enum
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Enum {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub name: Path,
    /// Optional underlying representation, e.g. `enum[u8] MyEnum`
    pub representation: Option<Type>,
    /// Combined generic params (bounds merged from param list + where clause)
    pub generic_params: Vec<GenericParameter>,
    /// `requires` clause
    pub requires: Vec<Type>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<Expression>,
}

// ============================================================================
// Union
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Union {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub name: Path,
    /// Combined generic params
    pub generic_params: Vec<GenericParameter>,
    /// `requires` clause
    pub requires: Vec<Type>,
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnionVariant {
    pub name: String,
    pub ty: Type,
}

// ============================================================================
// Struct
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Struct {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub is_packed: bool,
    pub name: Path,
    /// Combined generic params
    pub generic_params: Vec<GenericParameter>,
    /// `requires` clause
    pub requires: Vec<Type>,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
}

// ============================================================================
// Function
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Function {
    pub signature: FunctionSignature,
    pub body: Block,
}

/// Forward declaration (interface method, extern declaration)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionDeclaration {
    pub signature: FunctionSignature,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionSignature {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub is_extern: bool,
    /// `func!` – error-propagating function
    pub is_effect: bool,
    /// Receiver path segments for method implementations, e.g. `[Vec[T]]` in `func Vec[T]::push`.
    /// Each segment may carry generic arguments, e.g. `HashMap[K, V]::new`.
    pub receiver: Option<Vec<GenericPathSegment>>,
    pub name: String,
    /// Combined generic params (bounds merged from param list + where clause)
    pub outer_generic_params: Vec<GenericParameter>,
    pub func_generic_params: Vec<GenericParameter>,
    pub self_param: Option<SelfParameter>,
    pub params: Vec<FunctionParameter>,
    pub return_types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionParameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Block {
    pub statements: Vec<Statement>,
}

// ============================================================================
// Interfaces
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Interface {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub name: Path,
    /// Combined generic params
    pub generic_params: Vec<GenericParameter>,
    /// `extends` clause
    pub extends: Vec<Type>,
    pub methods: Vec<FunctionSignature>,
}

// ============================================================================
// Namespaces
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Namespace {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub name: Path,
    pub items: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NamespaceDeclaration {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub name: Path,
}

// ============================================================================
// Source File
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SourceFile {
    pub items: Vec<NamespaceItem>,
}

impl SourceFile {
    pub fn new(items: Vec<NamespaceItem>) -> Self {
        SourceFile { items }
    }

    pub fn items(&self) -> &Vec<NamespaceItem> {
        &self.items
    }
}

/// Top-level items at file or namespace scope
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NamespaceItem {
    Namespace(Namespace),
    NamespaceDeclaration(NamespaceDeclaration),
    Function(Function),
    FunctionDeclaration(FunctionDeclaration),
    TypeAlias(TypeAlias),
    Struct(Struct),
    Enum(Enum),
    Union(Union),
    Interface(Interface),
    Using(UsingStatement),
    Const(ConstStatement),
}

// ============================================================================
// Statements
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Statement {
    /// `pass`
    Pass,
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// standalone expression
    Expression(Box<Expression>),
    /// `let name: Type = value`
    Let(LetStatement),
    /// `mut name: Type = value`
    Mut(MutStatement),
    /// `const name: Type = value`
    Const(ConstStatement),
    /// `return expr`
    Return(Box<Expression>),
    /// `block name? { stmts }`
    Block(BlockStatement),
    /// `if cond { } elif ... else { }`
    If(IfStatement),
    /// `for pattern in iterable { }`
    For(ForStatement),
    /// `while cond { }`
    While(WhileStatement),
    /// `using path`
    Using(UsingStatement),
    // ── Nested definitions ──
    Function(Function),
    FunctionDeclaration(FunctionDeclaration),
    TypeAlias(TypeAlias),
    Struct(Struct),
    Enum(Enum),
    Union(Union),
    Interface(Interface),
    Namespace(Namespace),
}

// ── Statement structs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LetStatement {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub ty: Option<Type>,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MutStatement {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub ty: Option<Type>,
    pub value: Box<Expression>,
}

/// One segment of a const's qualified name, e.g. `namespacea` (no args) or `Option[T]` (with args).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConstPathSegment {
    pub name: String,
    pub generic_args: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConstStatement {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    /// Optional generic parameters declared directly on the const: `const[T, U] ...`.
    /// These become universally-quantified type variables available in the receiver and type.
    pub generic_params: Vec<GenericParameter>,
    /// Receiver path segments before the final name, e.g.
    ///   `namespacea::namespaceb::Option[T]` in
    ///   `const[T] namespacea::namespaceb::Option[T]::SOME_CONSTANT: i32 = 10`.
    /// Each segment carries its own optional generic arguments.
    pub receiver: Vec<ConstPathSegment>,
    pub name: String,
    pub ty: Option<Type>,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlockStatement {
    pub name: Option<String>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IfStatement {
    pub condition: Box<Expression>,
    pub then_body: Block,
    pub elif_clauses: Vec<ElifClause>,
    pub else_body: Option<Block>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElifClause {
    pub condition: Box<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ForStatement {
    pub pattern: String,
    pub iterable: Box<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WhileStatement {
    pub condition: Box<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UsingStatement {
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub path: Path,
}

// ============================================================================
// Helpers
// ============================================================================

/// Merge where-clause constraints into a list of generic parameters.
///
/// For each `GenericParameter::Type { name, bounds }` in `where_clause`:
/// - If a parameter with the same name already exists, append the bounds to it.
/// - Otherwise, insert a new entry.
pub fn merge_where_clause(
    mut params: Vec<GenericParameter>,
    where_clause: Vec<GenericParameter>,
) -> Vec<GenericParameter> {
    for constraint in where_clause {
        if let GenericParameter::Type { name, bounds, .. } = constraint {
            if let Some(existing) = params
                .iter_mut()
                .find(|p| matches!(p, GenericParameter::Type { name: n, .. } if *n == name))
            {
                if let GenericParameter::Type {
                    bounds: existing_bounds,
                    ..
                } = existing
                {
                    existing_bounds.extend(bounds);
                }
            } else {
                params.push(GenericParameter::Type {
                    name,
                    bounds,
                    default_type: None,
                });
            }
        }
    }
    params
}
