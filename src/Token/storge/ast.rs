use crate::import::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedExpr {
    pub expr: Expr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedStmt {
    pub stmt: Stmt,
    pub span: SourceSpan,
}



#[derive(Debug, Clone, PartialEq)]
pub struct GlobalConst {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int { bits: usize, signed: bool },
    Float { bits: usize },
    Bool,
    Char { bits: usize, signed: bool },
    Void,
    Ptr(Box<Type>),
    RawPtr(Box<Type>),
    Str { len_type: Box<Type> },
    StrSlice { char_type: Box<Type>, length_type: Box<Type> },
    Struct { name: String },
    Array { element: Box<Type>, size: Option<usize> },
    MultiArray { element: Box<Type>, dimensions: Vec<usize> },
    Tuple { fields: Vec<Type> },
    Union { variants: Vec<Type> },
    FnPtr { params: Vec<Type>, return_type: Box<Type> },
    Option { inner: Box<Type> },
    Result { ok: Box<Type>, err: Box<Type> },
    Intersection { types: Vec<Type> },
    HashMap { key: Box<Type>, value: Box<Type> },
    TripleDot,
    Variadic,
    SelfType,
    Any,
    Trait,
    Owned(Box<Type>),
    Ref(Box<Type>),
    MutRef(Box<Type>),
    Const(Box<Type>),
    ConstStr,
    StdStr,
    Auto 
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariant {
    Simple(String),
    Tuple(String, Vec<Type>),
    Struct(String, Vec<StructField>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct TokenError {
    pub error_type: ErrorType,
    pub span: SourceSpan,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    UnexpectedToken,
    InvalidSyntax,
    UnknownSymbol,
    TypeError,
    UndefinedReference,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        span: SourceSpan,
    },
    MissingToken {
        token: String,
        span: SourceSpan,
    },
    InvalidSyntax {
        message: String,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone)]
pub struct ParseDiagnostic {
    pub message: String,
    pub span: SourceSpan,
    pub severity: DiagnosticSeverity,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamModifier {
    Immutable,
    Mutable,
    Reference,
    MutableReference,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelfModifier {
    Immutable,
    Mutable,
    Borrow,
    Reference,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type, ParamModifier)>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunctionBody {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub is_public: bool,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExternDecl {
    Single {
        abi: String,
        func: ExternFunction,
    },
    Block {
        abi: String,
        library: String,
        functions: Vec<ExternFunction>,
    },
    SingleWithBody {
        abi: String,
        func: ExternFunctionBody,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub is_public: bool,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<(String, Type, ParamModifier)>,
    pub return_type: Type,
    pub self_modifier: Option<SelfModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: String,
    pub methods: Vec<TraitMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethod {
    pub name: String,
    pub params: Vec<(String, Type, ParamModifier)>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub self_modifier: Option<SelfModifier>,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub struct_name: String,
    pub trait_name: Option<String>,
    pub constructor_params: Vec<(String, Type)>,
    pub constructor_body: Option<Vec<(String, Expr)>>,
    pub methods: Vec<ImplMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub value: Expr,
    pub body: Vec<Stmt>,
}



#[derive(Debug, Clone, PartialEq)]
pub enum CastTarget {
    Type(Type),
    LibraryCall(String, Vec<Expr>),
    LibraryCallTyped(String, Vec<Type>),
    LibraryModuleCall(String, String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    TypedDeclaration {
        name: String,
        ty: Type,
        value: Expr,
        is_mutable: bool,
    },
    TupleUnpack {
        names: Vec<String>,
        value: Expr,
    },
    Assign(String, Expr),
    CompoundAssign(String, String, Expr),
    IndexAssign(Box<Expr>, Vec<Expr>, Expr),
    MemberAssign(Box<Expr>, String, Expr),
    ModuleAssign(String, String, Expr),
    ModuleCompoundAssign(String, String, String, Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    For(String, Expr, Vec<Stmt>),
    MemberCompoundAssign(Box<Expr>, String, String, Box<Expr>),
    Loop(Vec<Stmt>),
    Match(Expr, Vec<MatchCase>, Option<Vec<Stmt>>),
    Call(String, Vec<Expr>),
    ModuleCall(String, String, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    MethodCallNamed(Box<Expr>, String, Vec<(String, Expr)>),
    StaticMethodCall(String, String, Vec<Expr>),
    StaticMethodCallNamed(String, String, Vec<(String, Expr)>),
    Return(Option<Expr>),
    Break,
    Continue,
    Unsafe(Vec<Stmt>),
    Scope(Vec<Stmt>),
    StructDef(StructDef),
    EnumDef(EnumDef),
    ImplBlock(ImplBlock),
    TraitDef(TraitDef),
    ExternDecl(ExternDecl),
    ModuleImport(ModuleImport),
    ModuleUse(ModuleUse),
    ModuleDef {
        name: String,
        body: Vec<Stmt>,
        is_public: bool,
    },
    Function(Function),
    IfLet {
        pattern: Expr,
        value: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleImport {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleUse {
    pub module_name: String,
}

#[derive(Debug, Clone)]
pub enum ImportDecl {
    FileImport { name: String, from: String },
    LibraryImport { name: String },
    WildcardImport { from: String },
}

#[derive(Debug, Clone)]
pub struct ImportedModule {
    pub name: String,
    pub dll_path: PathBuf,
    pub ll_path: PathBuf,
    pub public_functions: Vec<LibraryFunction>,
    pub public_classes: Vec<ImportedClass>,
}

#[derive(Debug, Clone)]
pub struct ImportedClass {
    pub name: String,
    pub methods: Vec<LibraryFunction>,
}

#[derive(Debug, Clone)]
pub struct LibraryFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub abi: String,
}

#[derive(Debug, Clone)]
pub struct LibraryMetadata {
    pub exported_functions: Vec<FunctionSignature>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub constants: Vec<GlobalConst>,
    pub modules: Vec<Stmt>,  
}

#[derive(Debug, Clone)]
pub struct ExternFunctionMap {
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub abi: String,
    pub library: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExternFunctionInfo {
    pub abi: String,
    pub library: Option<String>,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub fields: Vec<(String, Type, bool)>,
    pub llvm_type: String,
}


pub struct ClassDef {
    pub name: String,
    pub methods: Vec<Function>,
    pub fields: Vec<(String, Type)>, 
}

#[derive(Debug, Clone)]
pub struct UndefinedFunction {
    pub name: String,
    pub call_location: SourceSpan,
    pub args_count: usize,
}

#[derive(Debug, Clone)]
pub struct UndefinedFunctions {
    pub library_functions: Vec<UndefinedFunction>,
}



 
pub struct IR {
    pub headers: String,
    pub type_definitions: String,
    pub forward_decls: String,
    pub function_decls: String,       
    pub helper_functions: String,      
    pub functions: String,
    pub added_typedefs: HashSet<String>,
    pub added_function_decls: HashSet<String>,
}

pub struct Codegen {
    pub config: CodegenConfig,
    pub type_registry: TypeRegistry,
    pub impl_methods: HashMap<(String, String), (Vec<(String, Type)>, Type, bool)>,
    pub c_code: String,
    pub globals: String,
    pub var_count: usize,
    pub label_count: usize,
    pub vars: HashMap<String, (String, Type)>,
    pub owned_vars: HashSet<String>,
    pub extern_functions: HashMap<String, ExternFunctionMap>,
    pub extern_block: HashMap<String, ExternFunctionMap>,
    pub structs: HashMap<String, StructInfo>,
    pub module_vars: HashMap<(String, String), (String, Type, bool)>,
    pub module_functions: HashMap<(String, String), (Vec<(String, Type)>, Type, bool)>,
    pub compilation_mode: CompilationMode,
    pub exported_functions: Vec<String>,
    pub externs_bodies: HashSet<String>,
    pub unsafe_depth: usize,
    pub scope_depth: usize,
    pub user_functions: HashMap<String, (Vec<(String, Type)>, Type)>,
    pub ir: IR,
    pub arch: ArchConfig,
    pub diagnostics: DiagnosticHandler,
    pub source_code: String,
    pub current_file: String,
    pub linked_libraries: Vec<String>,
    pub module_function_signatures: HashMap<String, String>,  
    pub module_init_functions: Vec<String>,
    pub current_return_type: Option<Type>,
}

pub struct CodegenConfig {
    pub arch: ArchConfig,
    pub optimization_level: OptimizationLevel,
    pub debug_info: bool,
}


#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum CompilationMode {
    #[default]
    Executable,
    Library,
}


#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum OptimizationLevel {
    #[default]
    None,
    O1,
    O2,
    O3,
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub spans: Vec<SourceSpan>,
    pub pos: usize,
    pub source: Arc<String>,
    pub diags: Vec<ParseDiagnostic>,
}

pub struct Lexer {
    pub input: Vec<char>,
    pub pos: usize,
    pub spans: Vec<SourceSpan>,
    pub errors: Vec<TokenError>,
    pub source_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub params: Vec<(String, Type, ParamModifier)>,
    pub return_type: Type,
}