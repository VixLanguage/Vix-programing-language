use crate::import::*;

impl Expr {
    pub fn location(&self) -> SourceLocation {
        SourceLocation {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
            length: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionWithLocation {
    pub name: String,
    pub params: Vec<(String, Type, ParamModifier)>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub is_public: bool,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefWithLocation {
    pub name: String,
    pub fields: Vec<StructField>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlockWithLocation {
    pub struct_name: String,
    pub trait_name: Option<String>,
    pub constructor_params: Vec<(String, Type)>,
    pub constructor_body: Option<Vec<(String, Expr)>>,
    pub methods: Vec<ImplMethod>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethodWithLocation {
    pub name: String,
    pub params: Vec<(String, Type, ParamModifier)>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub self_modifier: Option<SelfModifier>,
    pub location: SourceLocation,
}

pub fn default_location() -> SourceLocation {
    SourceLocation {
        file: "<unknown>".to_string(),
        line: 0,
        column: 0,
        length: 1,
    }
}

impl Default for Function {
    fn default() -> Self {
        Self {
            name: String::new(),
            params: Vec::new(),
            return_type: Type::Void,
            body: Vec::new(),
            is_public: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    ResultOk(Box<Expr>),
    ResultErr(Box<Expr>),
    ArrayGet {obj: Box<Expr>, reference: Box<Expr>,},
    Filter {obj: Box<Expr>, reference: Box<Expr>,},
    Not(Box<Expr>),
    Wait(Box<Expr>),
    Unwrap(Box<Expr>),
    UnwrapOr(Box<Expr>, Box<Expr>),
    Chars(Box<Expr>),
    Have { obj: Box<Expr>, item: Box<Expr> },
    IsNotEmpty(Box<Expr>),
    ContainAll { obj: Box<Expr>, items: Vec<Expr> },
    Contain { obj: Box<Expr>, item: Box<Expr> },
    Collect(Box<Expr>),
    IndexOf { obj: Box<Expr>, item: Box<Expr> },
    IsEmpty(Box<Expr>),
    Number(i32),
    Float(f32),
    String(String),
    Bool(bool),
    HexNumber(i32),
    BinaryNumber(i32),
    OctalNumber(i32),
    Char(i32),
    None,
    Some(Box<Expr>),
    Var(String),
    Call(String, Vec<Expr>),
    CallNamed(String, Vec<(String, Expr)>),
    FuncAddr(String),
    BinOp(String, Box<Expr>, Box<Expr>),
    UnOp(String, Box<Expr>),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Index(Box<Expr>, Vec<Expr>),
    TupleAccess(Box<Expr>, usize),
    MemberAccess(Box<Expr>, String),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    MethodCallNamed(Box<Expr>, String, Vec<(String, Expr)>),
    StaticMethodCall(String, String, Vec<Expr>),
    StaticMethodCallNamed(String, String, Vec<(String, Expr)>),
    ModuleAccess(String, String),
    ModuleCall(String, String, Vec<Expr>),
    ModuleCallNamed(String, String, Vec<(String, Expr)>),
    StructInit(String, Vec<(String, Expr)>),
    Cast(Box<Expr>, CastTarget),
    ReferenceTo(Type),
    Pipe(Box<Expr>, Box<Expr>),
    SizeOf(Type),
    AlignOf(Type),
    TypeOf(Box<Expr>),
    OffsetOf { struct_type: String, field: String },
    OneOf(Vec<Expr>),
    ArrayMethod { obj: Box<Expr>, method: String, args: Vec<Expr> },
    OptionMethod { obj: Box<Expr>, method: String, args: Vec<Expr> },
    Panic(Box<Expr>),
    Type(Type),
}