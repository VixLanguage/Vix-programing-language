use crate::import::*;

#[derive(Debug, Clone)]
pub struct ImportContext {
    pub library_imports: HashSet<String>,
    pub symbol_imports: HashMap<String, String>,
    pub module_exports: HashMap<String, Vec<String>>,
    pub auto_imported: bool,
}

impl Default for ImportContext {
    fn default() -> Self {
        Self::new()
    }
}

pub type ParseResult = (
    Program,
    Vec<StructDef>,
    Vec<EnumDef>,
    Vec<ExternDecl>,
    Vec<ModuleImport>,
    Vec<ModuleUse>,
    Vec<ClassDef>,
    Vec<ImplBlock>,
    Vec<TraitDef>,
    UndefinedFunctions,
    Vec<ImportDecl>
);

impl ImportContext {
    pub fn new() -> Self {
        ImportContext {
            library_imports: HashSet::new(),
            symbol_imports: HashMap::new(),
            module_exports: HashMap::new(),
            auto_imported: false,
        }
    }
    
    pub fn is_library_function(&self, name: &str) -> bool {
        self.library_imports.contains(name)
    }
    
    pub fn is_imported_symbol(&self, name: &str) -> bool {
        self.symbol_imports.contains_key(name)
    }
    
    pub fn get_symbol_library(&self, name: &str) -> Option<&String> {
        self.symbol_imports.get(name)
    }

    pub fn add_core_imports(&mut self) {
        if !self.auto_imported {
            self.library_imports.insert("core".to_string());
            self.auto_imported = true;
        }
    }

    pub fn add_module_export(&mut self, module: String, function: String) {
        self.module_exports.entry(module).or_insert_with(Vec::new).push(function);
    }

    pub fn is_module_function(&self, name: &str) -> bool {
        self.module_exports.values().any(|funcs| funcs.contains(&name.to_string()))
    }
}


impl Parser {
    pub fn new(tokens: Vec<Token>, source: String, spans: Vec<SourceSpan>) -> Self {
        Parser { 
            tokens, 
            spans, 
            pos: 0, 
            source: Arc::new(source), 
            diags: Vec::new(),
            type_aliases: HashMap::new(),
        }
    }



    pub fn build_import_context(import_decls: &[ImportDecl]) -> ImportContext {
        let mut context = ImportContext::new();

        context.add_core_imports();
        
        for import in import_decls {
            match import {
                ImportDecl::LibraryImport { name } => {
                    context.library_imports.insert(name.clone());
                }
                ImportDecl::FileImport { name, from } => {    
                    context.symbol_imports.insert(name.clone(), from.clone());
                }
                ImportDecl::WildcardImport { from } => {
                    context.library_imports.insert(from.clone());
                }
            }
        }
        
        context
    }


    pub fn parse_attribute(&mut self) -> (String, Option<String>) {
         
        if self.current() == Token::Hash {
            self.expect(Token::Hash, vec![Token::LeftBracket]);
            self.expect(Token::LeftBracket, vec![Token::RightBracket]);
        } else if self.current() == Token::At {
            self.advance();
        }
        
        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else if let Token::TypeIdentifier(id) = self.current() {
            self.advance();
            id
        } else {
            "error".to_string()
        };
        
         
        let param = if self.current() == Token::LeftParen {
            self.advance();
            let value = if let Token::String(s) = self.current() {
                self.advance();
                Some(s)
            } else {
                None
            };
            self.expect(Token::RightParen, vec![Token::Extern]);
            value
        } else if self.current() == Token::RightBracket {
            self.expect(Token::RightBracket, vec![Token::Func]);
            None
        } else {
            None
        };
        
        (name, param)
    }
    pub fn get_location(&self, pos: usize) -> SourceLocation {
        let lines: Vec<&str> = self.source.lines().collect();
        let mut current_pos = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_len = line.len() + 1; 
            if current_pos + line_len > pos {
                return SourceLocation {
                    file: "input".to_string(),
                    line: line_num + 1,
                    column: pos - current_pos + 1,
                    length: 1,
                };
            }
            current_pos += line_len;
        }
        
        SourceLocation {
            file: "input".to_string(),
            line: lines.len(),
            column: 1,
            length: 1,
        }
    }

    
    pub fn parse_type(&mut self) -> Type {
        let mut is_union = false;
        let mut _is_intersection = false;
        let mut types = vec![self.parse_base_type()];

        loop {
            match self.current() {
                Token::BitwiseOr | Token::Or => {
                    is_union = true;
                    self.advance();
                    types.push(self.parse_base_type());
                }
                Token::BitwiseAnd | Token::And => {
                    _is_intersection = true;
                    self.advance();
                    types.push(self.parse_base_type());
                }
                _ => break,
            }
        }

        let mut final_type = if types.len() == 1 {
            types.into_iter().next().unwrap()
                } else if is_union {
                    Type::Union { variants: types }
                } else {
            Type::Intersection { types }
        };

        while self.current() == Token::LeftBracket {
            self.advance();
            if self.current() == Token::RightBracket {
                self.advance();
                final_type = Type::Array {
                    element: Box::new(final_type),
                    size: None,
                };
            } else if let Token::Number(size) = self.current() {
                self.advance();
                self.expect(Token::RightBracket, vec![Token::Comma, Token::End]);

                final_type = Type::Array {
                    element: Box::new(final_type),
                    size: Some(size as usize),
                };
            } else {
                break;
            }
        }
        
        final_type
    }

    fn parse_base_type(&mut self) -> Type {
        let is_const = if self.current() == Token::Const {
            self.advance();
            true
        } else {
            false
        };
        
        let current = self.current();

        let base_type = match current {
            Token::Selfish if self.peek(1) != Token::LeftParen => {
                self.advance();
                Type::SelfType
            }
            Token::Trait => {
                self.advance();
                Type::Trait
            }
            Token::LeftBracket => {
                self.advance();

                if matches!(self.current(), Token::RightBracket | Token::EOF) {
                    self.expect(Token::RightBracket, vec![Token::Semicolon, Token::End]);
                    return Type::Array { element: Box::new(Type::Void), size: None };
                }
                
                 
                let elem_type = self.parse_type();

                if self.current() == Token::Semicolon {
                     
                    self.advance();
                    if let Token::Number(size) = self.current() {
                        self.advance();
                        self.expect(Token::RightBracket, vec![Token::Semicolon, Token::End]);
                        return Type::Array {
                            element: Box::new(elem_type),
                            size: Some(size as usize),
                        };
                    }
                }
                
                self.expect(Token::RightBracket, vec![Token::Semicolon, Token::End]);
                Type::Array {
                    element: Box::new(elem_type),
                    size: None,
                }
            }

            Token::Option => {
                self.advance();
                
                let close_token = if self.current() == Token::LeftBracket {
                    self.advance();
                    Token::RightBracket
                } else {
                    self.expect(Token::LeftParen, vec![Token::RightParen]);
                    Token::RightParen
                };
                
                let inner = self.parse_type();
                self.expect(close_token, vec![Token::Comma, Token::End]);
                
                Type::option(inner)
            }

            Token::Result => {
                self.advance();
                
                let close_token = if self.current() == Token::LeftBracket {
                    self.advance();
                    Token::RightBracket
                } else {
                    self.expect(Token::LeftParen, vec![Token::RightParen]);
                    Token::RightParen
                };

                let ok_type = self.parse_type();
                self.expect(Token::Comma, vec![Token::RightParen, Token::RightBracket]);
                let err_type = self.parse_type();

                self.expect(close_token, vec![Token::Comma, Token::End]);
                
                Type::result(ok_type, err_type)
            }

            Token::TypeIdentifier(type_name) => {
                self.advance();
                let base_type = Parser::parse_type_identifier(&type_name);
                let mut dimensions = Vec::new();
                while self.current() == Token::LeftBracket {
                    self.advance();
                    if let Token::Number(size) = self.current() {
                        dimensions.push(size as usize);
                        self.advance();
                        self.expect(Token::RightBracket, vec![Token::LeftBracket]);
                    } else {
                        return Type::Void;
                    }
                }
                if !dimensions.is_empty() {
                    Type::MultiArray {
                        element: Box::new(base_type),
                        dimensions,
                    }
                } else {
                    base_type
                }
            }
            Token::Caret => {
                self.advance();
                let inner = self.parse_base_type();
                Type::RawPtr(Box::new(inner))
            }
            Token::TripleDot => { self.advance(); Type::TripleDot }
            Token::Any => { self.advance(); Type::Any }
            Token::Tilde => {
                self.advance();
                Type::Owned(Box::new(self.parse_base_type()))
            }
            Token::Ampersand => {
                self.advance();
                if self.current() == Token::Mut {
                    self.advance();
                    Type::MutRef(Box::new(self.parse_base_type()))
                } else {
                    Type::Ref(Box::new(self.parse_base_type()))
                }
            }
            Token::Identifier(type_name) => {
                self.advance();
                if self.current() == Token::LeftBracket {
                    self.advance();
                    if self.current() == Token::RightBracket {
                        self.advance();
                        return Type::Array {
                            element: Box::new(Type::Struct { name: type_name }),
                            size: None,
                        };
                    } else if let Token::Number(size) = self.current() {
                        self.advance();
                        self.expect(Token::RightBracket, vec![]);
                        return Type::Array {
                            element: Box::new(Type::Struct { name: type_name }),
                            size: Some(size as usize),
                        };
                    }
                }
                Type::Struct { name: type_name }
            }
            Token::Bool => { self.advance(); Type::Bool }
            Token::Void => { self.advance(); Type::Void }
            Token::StdStr => { self.advance(); Type::StdStr }
            Token::Usize => { self.advance(); Type::Usize }
            Token::Str => {
                self.advance();
                
                if self.current() == Token::LeftParen {
                    self.advance();
                    let len_type = self.parse_type();
                    self.expect(Token::RightParen, vec![Token::Comma, Token::BitwiseOr]);
                    Type::Str { len_type: Box::new(len_type) }
                        } else if is_const {
                            Type::ConstStr
                            } else {
                            Type::Str { len_type: Box::new(Type::i64()) }
                        }
            }
            
            Token::BitwiseAnd => {
                self.advance();

                Type::Ptr(Box::new(self.parse_base_type()))
            }

            Token::LeftParen => {
                self.advance();

                let mut types = vec![self.parse_type()];

                if self.current() == Token::RightParen {
                    self.advance();

                    return Type::Tuple { fields: vec![] };
                } else if self.current() == Token::Comma {
                   
                    while self.current() == Token::Comma {
                        self.advance();
                        if self.current() == Token::RightParen { 
                            break; 
                        }

                        types.push(self.parse_type());
                    }

                    self.expect(Token::RightParen, vec![Token::Comma, Token::BitwiseOr]);

                    if self.current() == Token::LeftBracket {
                        self.advance();
                        if let Token::Number(size) = self.current() {
                            self.advance();
                            self.expect(Token::RightBracket, vec![Token::Comma]);
                            return Type::Array {
                                element: Box::new(Type::Tuple { fields: types }),
                                size: Some(size as usize),
                            };
                                } else if self.current() == Token::RightBracket {
                            self.advance();
                            return Type::Array {
                                element: Box::new(Type::Tuple { fields: types }),
                                size: None,
                            };
                        }
                    }
                    Type::Tuple { fields: types }
                } else if matches!(self.current(), Token::BitwiseOr | Token::Or) {
                    let mut types = vec![self.parse_type()];
                    while matches!(self.current(), Token::BitwiseOr | Token::Or) {
                        self.advance();
                        types.push(self.parse_base_type());
                    }
                    self.expect(Token::RightParen, vec![Token::LeftBracket]);
                    let union_type = Type::Union { variants: types };
                    if self.current() == Token::LeftBracket {
                        self.advance();
                        if let Token::Number(size) = self.current() {
                            self.advance();
                            self.expect(Token::RightBracket, vec![Token::Comma]);
                            return Type::Array {
                                element: Box::new(union_type),
                                size: Some(size as usize),
                            };
                        } else if self.current() == Token::RightBracket {
                            self.advance();
                            return Type::Array {
                                element: Box::new(union_type),
                                size: None,
                            };
                        }
                    }
                    union_type
                } else {
                    self.expect(Token::RightParen, vec![Token::Comma, Token::BitwiseOr]);
                    if self.current() == Token::LeftBracket {
                        self.advance();
                        if let Token::Number(size) = self.current() {
                            self.advance();
                            self.expect(Token::RightBracket, vec![Token::Comma]);
                            return Type::Array {
                                element: Box::new(self.parse_type()),
                                size: Some(size as usize),
                            };
                        } else if self.current() == Token::RightBracket {
                            self.advance();
                            return Type::Array {
                                element: Box::new(self.parse_type()),
                                size: None,
                            };
                        }
                    }
                    self.parse_type()
                }
            }
            _ => {
                self.advance();
                Type::Void
            }
        };
        
        if is_const && !matches!(base_type, Type::ConstStr) {
            Type::Const(Box::new(base_type))
                } else {
            base_type
        }
    }

    fn parse_primary(&mut self) -> Expr {
        let current = self.current();

        match current {
             Token::Plan => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                
                let format_str = if let Token::String(s) = self.current() {
                    let string = s.clone();
                    self.advance();
                    string
                } else {
                    String::new()
                };
                
                let mut args = Vec::new();
                if self.current() == Token::Comma {
                    self.advance();
                    while !matches!(self.current(), Token::RightParen | Token::EOF) {
                        args.push(self.parse_expr());
                        if self.current() == Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                
                self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
                Expr::Plan(format_str, args)
            }

            Token::String(s) => {
                let string_val = s.clone();
                self.advance();
                
                 
                if self.current() == Token::Dot {
                    self.advance();
                    
                    if let Token::Identifier(method_name) = self.current() {
                        let method = method_name.clone();
                        self.advance();
                        
                        if self.current() == Token::LeftParen {
                            self.advance();
                            
                             
                            let mut args = vec![Expr::String(string_val)];
                            
                            while !matches!(self.current(), Token::RightParen | Token::EOF) {
                                args.push(self.parse_expr());
                                if self.current() == Token::Comma {
                                    self.advance();
                                }
                            }
                            self.expect(Token::RightParen, vec![Token::Semicolon]);
                            return Expr::Call(method, args);
                        } else {
                             
                            return Expr::Call(method, vec![Expr::String(string_val)]);
                        }
                    }
                }
                
                Expr::String(string_val)
            }
            

            Token::Identifier(name) => {
                let var_name = name.clone();
                self.advance();
                
                 
                if let Some(aliased_expr) = self.type_aliases.get(&var_name) {
                     
                    return aliased_expr.clone();
                }
    
    if self.current() == Token::Dot {
        self.advance();
        
        if let Token::Identifier(next_name) = self.current() {
            let method_or_field = next_name.clone();
            self.advance();
            
            if self.current() == Token::LeftParen {
                self.advance();
                
                let mut args = Vec::new();
                while !matches!(self.current(), Token::RightParen | Token::EOF) {
                    args.push(self.parse_expr());
                    if self.current() == Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
                
                if var_name.chars().next().unwrap().is_uppercase() {
                    return Expr::StaticMethodCall(var_name, method_or_field, args);
                } else {
                    return Expr::MethodCall(
                        Box::new(Expr::Var(var_name)),
                        method_or_field,
                        args
                    );
                }
            } else {
                return Expr::MemberAccess(Box::new(Expr::Var(var_name)), method_or_field);
            }
        }
    } else if self.current() == Token::LeftParen {
        self.advance();
        
        if matches!(self.current(), Token::Identifier(_)) && self.peek(1) == Token::Equals {
            let mut named_args = Vec::new();
            while !matches!(self.current(), Token::RightParen | Token::EOF) {
                if let Token::Identifier(arg_name) = self.current() {
                    self.advance();
                    self.expect(Token::Equals, vec![Token::Comma, Token::RightParen]);
                    let arg_expr = self.parse_expr();
                    named_args.push((arg_name, arg_expr));
                    if self.current() == Token::Comma {
                        self.advance();
                    }
                }
            }
            self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
            return Expr::CallNamed(var_name, named_args);
        }
        
        let mut args = Vec::new();
        while !matches!(self.current(), Token::RightParen | Token::EOF) {
            args.push(self.parse_expr());
            if self.current() == Token::Comma {
                self.advance();
            }
        }
        self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
        
        return Expr::Call(var_name, args);
    }

    if var_name.chars().next().unwrap().is_uppercase() {
        return Expr::Call(var_name, vec![]);
    }
    Expr::Var(var_name)
}


            Token::Selfish => {
                self.advance();
                
                if self.current() == Token::Dot {
                    self.advance();
                    if let Token::Identifier(method_name) = self.current() {
                        let method = method_name.clone();
                        self.advance();
                        
                        if self.current() == Token::LeftParen {
                            self.advance();
                            let mut args = vec![Expr::Var("self".to_string())];
                            
                            while !matches!(self.current(), Token::RightParen | Token::EOF) {
                                args.push(self.parse_expr());
                                if self.current() == Token::Comma {
                                    self.advance();
                                }
                            }
                            self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
                            return Expr::MethodCall(Box::new(Expr::Var("self".to_string())), method, args[1..].to_vec());
                        } else {
                            return Expr::MemberAccess(Box::new(Expr::Var("self".to_string())), method);
                        }
                    }
                } else if self.current() == Token::LeftParen {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.current(), Token::RightParen | Token::EOF) {
                        args.push(self.parse_expr());
                        if self.current() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
                    return Expr::Call("self".to_string(), args);
                }
                
                Expr::Var("self".to_string())
            }

            Token::OneOf => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                let mut exprs = Vec::new();
                while !matches!(self.current(), Token::RightParen | Token::EOF) {
                    exprs.push(self.parse_expr());
                    if self.current() == Token::Comma {
                        self.advance();
                    }
                }
                self.expect(Token::RightParen, vec![Token::Semicolon]);
                Expr::OneOf(exprs)
            }
            Token::OffsetOf => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                let struct_name = if let Token::Identifier(name) = self.current() {
                    self.advance();
                    name
                } else {
                    self.advance();
                    "error".to_string()
                };
                self.expect(Token::Comma, vec![Token::RightParen]);
                let field_name = if let Token::Identifier(name) = self.current() {
                    self.advance();
                    name
                } else {
                    self.advance();
                    "error".to_string()
                };
                self.expect(Token::RightParen, vec![Token::Semicolon]);
                Expr::OffsetOf {
                    struct_type: struct_name,
                    field: field_name
                }
            }
            Token::AlignOf => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                let target_type = self.parse_type();
                self.expect(Token::RightParen, vec![Token::Semicolon]);
                Expr::AlignOf(target_type)
            }
            Token::TypeOf => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                let expr = self.parse_expr();
                self.expect(Token::RightParen, vec![Token::Semicolon]);
                Expr::TypeOf(Box::new(expr))
            }
            Token::None => {self.advance();Expr::None}
            Token::Number(n) => { self.advance(); Expr::Number(n) }
            Token::Char(ch) => {  self.advance(); Expr::Char(ch) }
            Token::Float(f) => { self.advance(); Expr::Float(f.into_inner()) }
            Token::HexNumber(n) => { self.advance(); Expr::HexNumber(n as i32) }
            Token::BinaryNumber(n) => { self.advance(); Expr::BinaryNumber(n as i32) }
            Token::OctalNumber(n) => { self.advance(); Expr::OctalNumber(n as i32) }
            Token::True => { self.advance(); Expr::Bool(true) }
            Token::False => { self.advance(); Expr::Bool(false) }
            Token::Some => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                let value = self.parse_expr();
                self.expect(Token::RightParen, vec![Token::Semicolon]);
                Expr::Some(Box::new(value))
            }
            
            Token::Ok => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                let value = self.parse_expr();
                self.expect(Token::RightParen, vec![Token::Semicolon]);
                Expr::ResultOk(Box::new(value))
            }
            
            Token::Err => {
                self.advance();
                self.expect(Token::LeftParen, vec![Token::RightParen]);
                let value = self.parse_expr();
                self.expect(Token::RightParen, vec![Token::Semicolon]);
                Expr::ResultErr(Box::new(value))
            }

            Token::LeftParen => {
                self.advance();
                if self.current() == Token::RightParen {
                    self.advance();
                    return Expr::Tuple(vec![]);
                }
                let first_expr = self.parse_expr();
                if self.current() == Token::Comma {
                    let mut elements = vec![first_expr];
                    while self.current() == Token::Comma {
                        self.advance();
                        if self.current() == Token::RightParen { break; }
                        elements.push(self.parse_expr());
                    }
                    self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
                    Expr::Tuple(elements)
                } else {
                    self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
                    first_expr
                }
            }
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !matches!(self.current(), Token::RightBracket | Token::EOF) {
                    elements.push(self.parse_expr());
                    if self.current() == Token::Comma {
                        self.advance();
                    }
                }
                self.expect(Token::RightBracket, vec![Token::Semicolon, Token::End]);
                Expr::Array(elements)
            }

            Token::LeftBrace => {
                self.advance();
                let mut entries = Vec::new();
                
                while !matches!(self.current(), Token::RightBrace | Token::EOF) {
                    self.expect(Token::LeftBracket, vec![Token::RightBracket]);
                    let key = self.parse_expr();
                    self.expect(Token::RightBracket, vec![Token::Equals]);
                    self.expect(Token::Equals, vec![Token::Comma, Token::RightBrace]);
                    let value = self.parse_expr();
                    
                    entries.push((key, value));
                    
                    if self.current() == Token::Comma {
                        self.advance();
                    }
                }
                
                self.expect(Token::RightBrace, vec![Token::Semicolon]);
                Expr::HashMap(entries)
            }

            _ => {
                self.advance();
                Expr::Number(0)
            }
        }
    }

    fn parse_post(&mut self, mut expr: Expr) -> Expr {
        loop {
            match self.current() {
                                 
                Token::Dot => {
                    self.advance();
                    
                    if let Token::Identifier(method_name) = self.current() {
                        let method = method_name.clone();
                        self.advance();
                        
                        if self.current() == Token::LeftParen {
                            self.advance();
                            
                            let mut args = Vec::new();
                            while !matches!(self.current(), Token::RightParen | Token::EOF) {
                                args.push(self.parse_expr());
                                if self.current() == Token::Comma {
                                    self.advance();
                                }
                            }
                            
                            self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
                            
                             
                            expr = Expr::MethodCall(Box::new(expr), method, args);
                        } else {
                             
                            expr = Expr::MemberAccess(Box::new(expr), method);
                        }
                    }
                }

                Token::LeftBrace => {
                    self.advance();
                    let mut entries = Vec::new();
                    
                    while !matches!(self.current(), Token::RightBrace | Token::EOF) {
                        self.expect(Token::LeftBracket, vec![Token::RightBracket]);
                        let key = self.parse_expr();
                        self.expect(Token::RightBracket, vec![Token::Equals]);
                        self.expect(Token::Equals, vec![Token::Comma, Token::RightBrace]);
                        let value = self.parse_expr();
                        
                        entries.push((key, value));
                        
                        if self.current() == Token::Comma {
                            self.advance();
                        }
                    }
                    
                    self.expect(Token::RightBrace, vec![Token::Semicolon]);
                    expr = Expr::HashMap(entries);
                }
                Token::LeftBracket => {
                    self.advance();
                    let start = self.parse_expr();
                    
                    if self.current() == Token::DoubleDot {
                        self.advance();
                        let end = self.parse_expr();
                        self.expect(Token::RightBracket, vec![Token::Dot, Token::LeftBracket]);
                        expr = Expr::Slice(Box::new(expr), Box::new(start), Box::new(end));
                    } else {
                        self.expect(Token::RightBracket, vec![Token::Dot, Token::LeftBracket]);
                        expr = Expr::Index(Box::new(expr), vec![start]);
                    }
                }
                _ => break,
            }
        }
        expr
    }

    pub fn parse_unary(&mut self) -> Expr {
        match self.current() {
            Token::Not => {
                self.advance();
                let expr = self.parse_unary();
                Expr::Not(Box::new(expr))
            }
            Token::Ampersand => {
                self.advance();
                if let Token::Identifier(name) = self.current() {
                    self.advance();
                    if self.current() == Token::LeftParen {
                        self.advance();
                        Expr::Number(0)
                    } else {
                        Expr::FuncAddr(name)
                    }
                } else {
                    let expr = self.parse_unary();
                    Expr::UnOp("&".to_string(), Box::new(expr))
                }
            }
            Token::Minus => {
                let op = match self.current() {
                    Token::Minus => "-".to_string(),
                    _ => unreachable!(),
                };
                self.advance();
                let expr = self.parse_unary();
                Expr::UnOp(op, Box::new(expr))
            }
            _ => {
                let primary = self.parse_primary();
                self.parse_post(primary)
            }
        }
    }

    pub fn is_type_token(&self, token: Token) -> bool {
        matches!(token,
            Token::TypeIdentifier(_) | Token::Bool | Token::Void | Token::Any |
            Token::Str | Token::Ampersand| Token::BitwiseAnd | Token::TripleDot |
            Token::Tilde | Token::Mut | Token::LeftParen | Token::LeftBracket |
            Token::Identifier(_) | Token::Option | Token::Result | Token::Selfish |
            Token::Trait | Token::Caret | Token::StdStr
        )
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_logic_or()
    }

    fn parse_logic_or(&mut self) -> Expr {
        let mut node = self.parse_logic_and();

        while self.current() == Token::Or { 
            self.advance();
            let right = self.parse_logic_and();
            node = Expr::BinOp("||".to_string(), Box::new(node), Box::new(right));
        }

        node
    }

    fn parse_logic_and(&mut self) -> Expr {
        let mut node = self.parse_bitwise_or();

        while self.current() == Token::And {
            self.advance();
            let right = self.parse_bitwise_or();
            node = Expr::BinOp("&&".to_string(), Box::new(node), Box::new(right));
        }

        node
    }

    fn parse_bitwise_or(&mut self) -> Expr {
        let mut node = self.parse_bitwise_xor();
        while self.current() == Token::Pipe {
             self.advance();
             let right = self.parse_bitwise_xor();
             node = Expr::BinOp("|".to_string(), Box::new(node), Box::new(right));
        }
        node
    }

    fn parse_bitwise_xor(&mut self) -> Expr {
        let mut node = self.parse_bitwise_and();
        while self.current() == Token::Caret {
             self.advance();
             let right = self.parse_bitwise_and();
             node = Expr::BinOp("^".to_string(), Box::new(node), Box::new(right));
        }
        node
    }

    fn parse_bitwise_and(&mut self) -> Expr {
        let mut node = self.parse_equality();
        while self.current() == Token::Ampersand {
             self.advance();
             let right = self.parse_equality();
             node = Expr::BinOp("&".to_string(), Box::new(node), Box::new(right));
        }
        node
    }

    fn parse_equality(&mut self) -> Expr {
        let mut node = self.parse_comparison();

        while matches!(self.current(), Token::EqualsEquals | Token::NotEquals) {
            let op = match self.current() {
                Token::EqualsEquals => "==".to_string(),
                Token::NotEquals => "!=".to_string(),
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison();
            node = Expr::BinOp(op, Box::new(node), Box::new(right));
        }

        node
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut node = self.parse_shift();

        while matches!(self.current(), Token::Less | Token::LessEquals | Token::Greater | Token::GreaterEquals) {
            let op = match self.current() {
                Token::Less => "<".to_string(),
                Token::LessEquals => "<=".to_string(),
                Token::Greater => ">".to_string(),
                Token::GreaterEquals => ">=".to_string(),
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_shift();
            node = Expr::BinOp(op, Box::new(node), Box::new(right));
        }

        node
    }

    fn parse_shift(&mut self) -> Expr {
        let mut node = self.parse_term();
        while matches!(self.current(), Token::LeftShift | Token::RightShift) {
             let op = match self.current() {
                Token::LeftShift => "<<".to_string(),
                Token::RightShift => ">>".to_string(),
                _ => unreachable!(),
            };
             self.advance();
             let right = self.parse_term();
             node = Expr::BinOp(op, Box::new(node), Box::new(right));
        }
        node
    }

    fn parse_term(&mut self) -> Expr {
        let mut node = self.parse_factor();

        while matches!(self.current(), Token::Plus | Token::Minus) {
            let op = match self.current() {
                Token::Plus => "+".to_string(),
                Token::Minus => "-".to_string(),
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_factor();
            node = Expr::BinOp(op, Box::new(node), Box::new(right));
        }

        node
    }

    fn parse_factor(&mut self) -> Expr {
        let mut node = self.parse_unary();

        while matches!(self.current(), Token::Star | Token::Slash | Token::Percent) {
            let op = match self.current() {
                Token::Star => "*".to_string(),
                Token::Slash => "/".to_string(),
                Token::Percent => "%".to_string(),
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary();
            node = Expr::BinOp(op, Box::new(node), Box::new(right));
        }

        node
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.current() {
            Token::Let => {
                self.advance();
                
                let name = if let Token::Identifier(n) = self.current() {
                    n
                } else {
                    panic!("Expected identifier after 'let'");
                };
                self.advance();

                let is_mutable = if self.current() == Token::Mut {
                    self.advance();
                    true
                } else {
                    false
                };

                let ty = if self.current() == Token::Colon {
                    self.advance();
                    self.parse_type()
                } else {
                    Type::Auto
                };

                self.expect(Token::Equals, vec![Token::Semicolon]);
                let value = self.parse_expr();

                Stmt::TypedDeclaration {
                    name,
                    ty,
                    value,
                    is_mutable,
                }
            }
        Token::Identifier(name) if self.peek(1) == Token::Colon => {
                let var_name = name.clone();
                self.advance();
                self.advance(); 
                
                let var_type = self.parse_type();
                self.expect(Token::Equals, vec![Token::Semicolon]);
                
                let value_expr = self.parse_expr();

                Stmt::TypedDeclaration {
                    name: var_name,
                    ty: var_type,
                    value: value_expr,
                    is_mutable: false,
                }

            }
                
            Token::Unsafe => {
                self.advance();
                self.expect(Token::Colon, vec![Token::End]);
               
                let mut body = Vec::new();
                while !matches!(self.current(), Token::End | Token::EOF) {
                    body.push(self.parse_stmt());
                    if self.current() == Token::Semicolon {
                        self.advance();
                    }
                }

                if self.current() == Token::End {
                    self.advance();
                }
               
                Stmt::Unsafe(body)
            }
            Token::Scope => {
                self.advance();
                self.expect(Token::Colon, vec![Token::End]);
               
                let mut body = Vec::new();
                while !matches!(self.current(), Token::End | Token::EOF) {
                    body.push(self.parse_stmt());
                    if self.current() == Token::Semicolon {
                        self.advance();
                    }
                }
               
                if self.current() == Token::End {
                    self.advance();
                }
               
                Stmt::Scope(body)
            }

            Token::Create => {
                self.advance();
                
                if let Token::Identifier(first_name) = self.current() {
                    self.advance();
                    
                    if self.current() == Token::Comma {
                        let mut names = vec![first_name];
                        while self.current() == Token::Comma {
                            self.advance();
                            if let Token::Identifier(name) = self.current() {
                                names.push(name);
                                self.advance();
                            }
                        }
                        
                        self.expect(Token::Equals, vec![Token::Semicolon]);
                        let value = self.parse_expr();
                        
                        Stmt::TupleUnpack {
                            names,
                            value,
                        }
                    } else {
                        let name = first_name;
                        self.expect(Token::Equals, vec![Token::Semicolon]);
                        let value = self.parse_expr();
                        
                        let ty = if self.current() == Token::As {
                            self.advance();
                            self.parse_type()
                        } else {
                            Parser::infer_type(&value)
                        };
                        
                        Stmt::TypedDeclaration {
                            name,
                            ty,
                            value,
                            is_mutable: false,
                        }
                    }
                } else {
                    self.advance();
                    Stmt::Break
                }
            }
            Token::Match => {
                self.advance();
                let match_expr = self.parse_expr();
                self.expect(Token::Colon, vec![Token::End, Token::Case, Token::Default]);
               
                let mut cases = Vec::new();
                let mut default_body = None;
               
                while !matches!(self.current(), Token::End | Token::EOF) {
                    if self.current() == Token::Default {
                        self.advance();
                        self.expect(Token::Colon, vec![Token::End]);
                        let mut stmts = Vec::new();
                        while !matches!(self.current(), Token::End | Token::EOF) {
                            stmts.push(self.parse_stmt());
                            if self.current() == Token::Semicolon {
                                self.advance();
                            }
                        }
                        default_body = Some(stmts);
                        break;
                    } else {
                        if self.current() == Token::Case {
                            self.advance();
                        }
                       
                        let case_value = self.parse_expr();
                        self.expect(Token::Colon, vec![Token::End, Token::Case, Token::Default]);
                       
                        let mut case_stmts = Vec::new();
                        while !matches!(self.current(), Token::Case | Token::Default | Token::End | Token::EOF) {
                            case_stmts.push(self.parse_stmt());
                            if self.current() == Token::Semicolon {
                                self.advance();
                            }
                        }

                        cases.push(MatchCase {
                            value: case_value,
                            body: case_stmts,
                        });
                    }
                }
               
                if self.current() == Token::End {
                    self.advance();
                }
               
                Stmt::Match(match_expr, cases, default_body)
            }
            Token::Return => {
                self.advance();
                let expr = if matches!(self.current(), Token::Semicolon | Token::End | Token::EOF) {
                    None
                } else {
                    Some(self.parse_expr())
                };
                Stmt::Return(expr)
            }
            Token::Break => {
                self.advance();
                Stmt::Break
            }
            Token::Continue => {
                self.advance();
                Stmt::Continue
            }
            Token::If => {
                self.advance();
                let expr = self.parse_expr();
                
                if self.current() == Token::Equals {
                    self.advance();
                    let value = self.parse_expr();
                    
                    if self.current() == Token::Then {
                        self.advance();
                    }
                    
                    let mut then_body = Vec::new();
                    while !matches!(self.current(), Token::Else | Token::End | Token::EOF) {
                        then_body.push(self.parse_stmt());
                        if self.current() == Token::Semicolon {
                             self.advance();
                        }
                    }
                    
                    let else_body = if self.current() == Token::Else {
                        self.advance();
                        self.expect(Token::Colon, vec![Token::End]);
                        let mut stmts = Vec::new();
                        while !matches!(self.current(), Token::End | Token::EOF) {
                            stmts.push(self.parse_stmt());
                            if self.current() == Token::Semicolon {
                                self.advance();
                            }
                        }
                        Some(stmts)
                    } else {
                        None
                    };
                    
                    if matches!(self.current(), Token::End | Token::Comma) {
                         self.advance();
                    }
                    
                    Stmt::IfLet {
                        pattern: expr,
                        value,
                        then_block: then_body,
                        else_block: else_body
                    }
                } else {
                    let mut then_body = Vec::new();
                    let mut stmts = Vec::new();
    
                    let cond = expr;
                    let else_body = if self.current() == Token::Else {
                        self.advance();
                        self.expect(Token::Colon, vec![Token::End]);
                        
                        while !matches!(self.current(), Token::End | Token::EOF) {
                            stmts.push(self.parse_stmt());
                            if self.current() == Token::Semicolon {
                                self.advance();
                            }
                        }
                        Some(stmts)
                    } else {
                        None
                    };
                   
                    if self.current() == Token::Then {
                        self.advance()
                    }
    
                    while !matches!(self.current(), Token::Else | Token::End | Token::EOF) {
                        then_body.push(self.parse_stmt());
    
                        if self.current() == Token::Semicolon {
                            self.advance();
                        }
                    }
    
                    if matches!(self.current(), Token::End | Token::Comma) {
                        self.advance();
                    }
    
                    Stmt::If(cond, then_body, else_body)
                }
            }

            Token::While => {
                self.advance();
                let cond = self.parse_expr();
                
                if self.current() == Token::Do {
                    self.advance();
                }

                let mut body = Vec::new();
                while !matches!(self.current(), Token::End | Token::EOF) {
                    body.push(self.parse_stmt());
                    if self.current() == Token::Semicolon {
                        self.advance();
                    }
                }
                
                if self.current() == Token::End {
                    self.advance();
                }
                Stmt::While(cond, body)
            }

            Token::For => {
                self.advance();

                let var_name = if let Token::Identifier(name) = self.current() {
                    self.advance();
                    name
                } else {
                    self.advance();
                    "error".to_string()
                };
                
                self.expect(Token::In, vec![Token::Do, Token::End]);
                let iter_expr = self.parse_expr();
                println!("[DEBUG] parse_for: iter_expr={:?}", iter_expr);
                self.expect(Token::Do, vec![Token::End]);
                
                let mut body = Vec::new();
                while !matches!(self.current(), Token::End | Token::EOF) {
                    println!("[DEBUG] parse_for: about to parse stmt at {:?}", self.current());
                    let s = self.parse_stmt();
                    println!("[DEBUG] parse_for: parsed stmt: {:?}", s);
                    body.push(s);
                    if self.current() == Token::Semicolon {
                        self.advance();
                    }
                }

                println!("[DEBUG] parse_for: loop body length={}", body.len());
                if self.current() == Token::End {
                    self.advance();
                }
                Stmt::For(var_name, iter_expr, body)
            }
            Token::Hash => {
                let mut attributes = Vec::new();
                while self.current() == Token::Hash {
                    let (name, param) = self.parse_attribute();
                     
                    attributes.push(if let Some(p) = param {
                        format!("{}({})", name, p)
                    } else {
                        name
                    });
                }
                
                let is_public = if self.current() == Token::Pub {
                    self.advance();
                    true
                } else {
                    false 
                };
                
                if self.current() == Token::Func {
                    Stmt::Function(self.parse_function_with_visibility(false, is_public, attributes))
                } else {
                    self.diags.push(ParseDiagnostic {
                        message: "Expected function after attributes".to_string(),
                        span: self.current_span(),
                        severity: DiagnosticSeverity::Error,
                        help: Some("Attributes are currently only supported on functions".to_string()),
                    });

                    self.advance();
                    Stmt::Expr(Expr::None)
                }
            }

            Token::Mod => self.parse_module(false),
            Token::Func => Stmt::Function(self.parse_function_with_visibility(false, false, vec![])),
            Token::Pub => {
                self.advance();
                match self.current() {
                     Token::Mod => self.parse_module(true),
                     Token::Func => Stmt::Function(self.parse_function_with_visibility(false, true, vec![])),
                     Token::Struct => Stmt::StructDef(self.parse_struct(true)),
                     Token::Enum => Stmt::EnumDef(self.parse_enum(true)),
                     _ => panic!("Expected Item after Pub"), 
                }
            },
            Token::Mut | Token::Mutable => {
                self.advance();
                if let Token::Identifier(name) = self.current() {
                    self.advance();
                    self.expect(Token::Colon, vec![Token::Equals]);
                    let ty = self.parse_type();
                    
                    self.expect(Token::Equals, vec![Token::Semicolon]);
                    let value = self.parse_expr();
                    
                    Stmt::TypedDeclaration {
                        name,
                        ty,
                        value,
                        is_mutable: true,
                    }
                } else {
                   Stmt::Expr(self.parse_expr()) 
                }
            }
            _ => {
                let expr = self.parse_expr();
               
                if self.current() == Token::Equals {
                    self.advance();
                    let value = self.parse_expr();
                   
                    match expr {
                        Expr::Var(name) => Stmt::Assign(name, value),
                        Expr::Index(obj, indices) => Stmt::IndexAssign(obj, indices, value),
                        Expr::MemberAccess(obj, field) => Stmt::MemberAssign(obj, field, value),
                        Expr::ModuleAccess(module, member) => Stmt::ModuleAssign(module, member, value),
                        _ => Stmt::Expr(expr)
                    }
                } else if self.current() == Token::Colon {
                    if let Expr::Var(name) = expr {
                        self.advance();

                        let ty = self.parse_type();
                        let value = if self.current() == Token::Equals {
                            self.advance();
                            self.parse_expr()

                                } else {
                            Expr::Number(0)
                        };

                        Stmt::TypedDeclaration {
                            name,
                            ty,
                            value,
                            is_mutable: false,
                        }
                    } else {
                         Stmt::Expr(expr)
                    }
                } else if let Some(op) = self.parse_compound_op() {
                    self.advance();
                   
                    match expr {
                        Expr::Var(name) => Stmt::CompoundAssign(name, op, self.parse_expr()),
                        Expr::ModuleAccess(module, member) => Stmt::ModuleCompoundAssign(module, member, op, self.parse_expr()),
                        _ => Stmt::Expr(expr)
                    }
                } else {
                    match expr {
                        Expr::Call(func, args) => Stmt::Call(func, args),
                        Expr::ModuleCall(module, func, args) => Stmt::ModuleCall(module, func, args),
                        _ => Stmt::Expr(expr)
                    }
                }
            }
        }
    }
    
    fn parse_struct(&mut self, is_public: bool) -> StructDef {
        self.expect(Token::Struct, vec![Token::Colon, Token::End]);
        
        let mut fields = Vec::new();
        let name = if let Token::Identifier(name) = self.current() {
            self.advance();
            name
        } else {
            self.advance();
            "error".to_string()
        };

        if self.current() == Token::Semicolon {
            self.advance();
            return StructDef { name, fields, is_public };
        }

        self.expect(Token::Colon, vec![Token::End]);
        
        while !matches!(self.current(), Token::End | Token::EOF) {
            let is_public = 
                if self.current() == Token::Pub {
                    self.advance();
                    true
                } else {
                    false
                };
            
            let is_mutable = 
                if self.current() == Token::Mutable || self.current() == Token::Mut {
                    self.advance();
                    true
                } else {
                    false
                };
            
            let field_name = 
                if let Token::Identifier(field_name) = self.current() {
                    self.advance();
                    field_name
                } else {
                    self.advance();
                    "error".to_string()
                };
            
            self.expect(Token::Equals, vec![Token::Comma, Token::End]);

            fields.push(StructField {
                name: field_name,
                ty: self.parse_type(),
                is_public,
                is_mutable,
            });
        }
        
        if self.current() == Token::End {
            self.advance();
        }
        
        StructDef { name, fields, is_public }
    }
    fn parse_enum(&mut self, is_public: bool) -> EnumDef {
    self.expect(Token::Enum, vec![Token::Colon, Token::End]);
            
    let mut variants = Vec::new();
    let name = if let Token::Identifier(name) = self.current() {
        self.advance();
        name
    } else {
        self.advance();
        "error".to_string()
    };
    
    self.expect(Token::Colon, vec![Token::End]);

    while !matches!(self.current(), Token::End | Token::EOF) {
        if let Token::Identifier(vname) = self.current() {
            self.advance();
            
            if self.current() == Token::LeftParen {
                self.advance();

                let is_struct_like = matches!(self.current(), Token::Identifier(_)) 
                    && self.peek(1) == Token::Colon;
                
                if is_struct_like {
                     
                    let mut fields = Vec::new();
                    while self.current() != Token::RightParen && self.current() != Token::EOF {
                        let fname = if let Token::Identifier(n) = self.current() { 
                            n 
                        } else { 
                            "err".to_string() 
                        };

                        self.advance();
                        self.expect(Token::Colon, vec![Token::Comma, Token::RightParen]);
                        
                        fields.push(StructField { 
                            name: fname, 
                            ty: self.parse_type(), 
                            is_public: true, 
                            is_mutable: true 
                        });

                        if self.current() == Token::Comma { 
                            self.advance(); 
                        }
                    }
                    self.expect(Token::RightParen, vec![Token::End]);
                    variants.push(EnumVariant::Struct(vname, fields));
                } else {
                     
                    let mut types = Vec::new();
                    while self.current() != Token::RightParen && self.current() != Token::EOF {
                        types.push(self.parse_type());
                        if self.current() == Token::Comma { 
                            self.advance(); 
                        }
                    }
                    self.expect(Token::RightParen, vec![Token::End]);
                    
                     
                    variants.push(EnumVariant::Tuple(vname, types));
                }
            } else {
                 
                variants.push(EnumVariant::Simple(vname));
            }
            
            if self.current() == Token::Comma { 
                self.advance(); 
            }
        } else {
            self.advance();
        }
    }
    
    if self.current() == Token::End {
        self.advance();
    }
    
    EnumDef { name, variants, is_public }
}

    fn parse_function(&mut self, is_module: bool) -> Function {
        self.parse_function_with_visibility(is_module, false, vec![])
    }
   
    
    fn parse_function_with_visibility(&mut self, is_module: bool, is_public: bool, attributes: Vec<String>) -> Function {
        self.expect(Token::Func, vec![Token::Colon, Token::End]);
        
        let name = if let Token::Identifier(name) = self.current() {
            self.advance();
            name
        } else {
            self.advance();
            "error".to_string()
        };
        
        self.expect(Token::LeftParen, vec![Token::RightParen, Token::Arrow, Token::Colon]);
        
        let mut params = Vec::new();
        
        let is_self_param = match self.current() {
            Token::Selfish => true,
            Token::Mutable | Token::Mut if self.peek(1) == Token::Selfish => true,
            Token::Reference | Token::Ampersand if self.peek(1) == Token::Selfish => true,
            Token::Reference | Token::Ampersand 
                if matches!(self.peek(1), Token::Mutable | Token::Mut) 
                && self.peek(2) == Token::Selfish => true,
            _ => false,
        };

        if is_self_param {
            while matches!(
                self.current(), 
                Token::Selfish | Token::Mutable | Token::Mut | Token::Reference | Token::Ampersand
            ) {
                self.advance();
            }

            if self.current() == Token::Comma {
                self.advance();
            }
        }

        while self.current() != Token::RightParen {
            let modifier = match self.current() {
                Token::Mutable => {
                    self.advance();
                    if self.current() == Token::Reference {
                        self.advance();
                        ParamModifier::MutableReference
                    } else {
                        ParamModifier::Mutable
                    }
                }
                Token::Reference => {
                    self.advance();
                    ParamModifier::Reference
                }
                _ => ParamModifier::Immutable
            };
            
            let pname = if let Token::Identifier(pname) = self.current() {
                self.advance();
                pname
            } else {
                self.advance();
                "error".to_string()
            };
            
            self.expect(Token::Colon, vec![Token::Comma, Token::RightParen]);
            let ptype = self.parse_type();
            params.push((pname, ptype, modifier));
            
            if self.current() == Token::Comma {
                self.advance();
            }
        }
        self.expect(Token::RightParen, vec![Token::Colon]);
        
        let mut body = Vec::new();
        let return_type = 
            if self.current() == Token::Arrow || self.current() == Token::Colon {
                self.advance();
           
                if self.is_type_token(self.current()) {
                    self.parse_type()
                        } else {
                    Type::Void
                }
            } else if name == "main" && !is_module {
            Type::i32()
                } else {
                Type::Void
            };
       
        
        if self.current() == Token::Colon {
            self.advance();
        }

        while !matches!(self.current(), Token::End | Token::EOF) {
            body.push(self.parse_stmt());
            if self.current() == Token::Semicolon {
                self.advance();
            }
        }
       
        self.expect(Token::End, vec![]);
       
        Function {
            name,
            params,
            return_type,
            body,
            is_public,
            attributes,
        }
    }

    fn parse_import(&mut self) -> ImportDecl {
         
        if self.current() == Token::From {
            self.advance();
            if let Token::Identifier(from) = self.current() {
                self.advance();
                self.expect(Token::Import, vec![Token::Star]);
                self.expect(Token::Star, vec![Token::Semicolon]);
                return ImportDecl::WildcardImport { from };
            }
        }

        self.expect(Token::Import, vec![Token::Semicolon, Token::From, Token::Star]);
        
         
        if self.current() == Token::Star {
             self.advance();
             self.expect(Token::From, vec![Token::Identifier("".to_string())]);
             if let Token::Identifier(from) = self.current() {
                 self.advance();
                 return ImportDecl::WildcardImport { from };
             }
        }

        if let Token::Identifier(name) = self.current() {
            self.advance();
            
            if self.current() == Token::From {
                self.advance();
                
                if let Token::Identifier(from) = self.current() {
                    self.advance();
                    return ImportDecl::FileImport { name, from };
                }
            }
            
            return ImportDecl::LibraryImport { name };
        }
        
        ImportDecl::LibraryImport { name: "unknown".to_string() }
    }
    fn parse_module(&mut self, is_public: bool) -> Stmt {
        self.expect(Token::Mod, vec![Token::Identifier("".to_string())]);
        
        let name = if let Token::Identifier(name) = self.current() {
            self.advance();
            name
        } else {
             self.advance();
            "error".to_string()
        };
        
        self.expect(Token::LeftBrace, vec![Token::End]);
        
        let mut body = Vec::new();
        while !matches!(self.current(), Token::RightBrace | Token::EOF) {
             println!("DEBUG: Module body token: {:?}", self.current());
             body.push(self.parse_stmt());
             if self.current() == Token::Semicolon {
                 self.advance();
             }
        }
        
        println!("DEBUG: Expecting RightBrace");
        self.expect(Token::RightBrace, vec![Token::Semicolon]);
        println!("DEBUG: Finished parsing module");
        
        Stmt::ModuleDef {
            name,
            body,
            is_public
        }
    }

    fn parse_type_alias(&mut self) -> Stmt {
         
        self.expect(Token::Type, vec![Token::Identifier("".to_string())]);
        
        let name = if let Token::Identifier(name) = self.current() {
            self.advance();
            name
        } else {
            self.advance();
            "error".to_string()
        };
        
         
        self.expect(Token::Equals, vec![Token::Semicolon]);
        
         
        let target = Box::new(self.parse_expr());
        
         
        self.type_aliases.insert(name.clone(), *target.clone());
        
        Stmt::TypeAlias { name, target }
    }

    fn parse_module_exports(&mut self) -> Stmt {
         
        if let Token::Identifier(id) = self.current() {
            if id != "export" {
                panic!("Expected 'export' identifier");
            }
            self.advance();
        }
        
         
        self.expect(Token::LeftParen, vec![Token::RightParen]);
        
        let mut exports = Vec::new();
        
        while self.current() != Token::RightParen && self.current() != Token::EOF {
             
            let name = if let Token::Identifier(name) = self.current() {
                self.advance();
                name
            } else {
                self.advance();
                "error".to_string()
            };
            
             
            let alias = if let Token::Identifier(as_kw) = self.current() {
                if as_kw == "as" {
                    self.advance();
                    if let Token::Identifier(alias_name) = self.current() {
                        self.advance();
                        Some(alias_name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            
            exports.push((name, alias));
            
             
            if self.current() == Token::Comma {
                self.advance();
            }
        }
        
         
        self.expect(Token::RightParen, vec![Token::Semicolon]);
        
        Stmt::ModuleExports { exports }
    }

    

    pub fn parse(mut self) -> ParseResult {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut externs = Vec::new();
        let mut imports = Vec::new();
        let mut uses = Vec::new();
        let classes = Vec::new();
        let mut impls = Vec::new();
        let traits = Vec::new();
        let mut import_decls = Vec::new();
        let mut constants = Vec::new();
        let mut modules = Vec::new();
        let _stmts: Vec<Stmt> = Vec::new(); 
        
         
         
       
         
        while self.current() != Token::EOF {
            if self.current() == Token::Import || self.current() == Token::From {
                import_decls.push(self.parse_import());
            } else {
                break;
            }
        }
        
        let import_context = Self::build_import_context(&import_decls);
        
        while self.current() != Token::EOF {
            println!("DEBUG: Current token: {:?}", self.current());
            match self.current() {
                Token::Import => { self.advance(); }
                
                Token::Pub => {
                    self.advance();
                    println!("DEBUG: Inside Pub");
                    match self.current() {
                        Token::Mod => {
                             println!("DEBUG: Parsing public module");
                             modules.push(self.parse_module(true));
                        }
                        Token::Func => {
                            println!("DEBUG: Parsing public function");
                            functions.push(self.parse_function_with_visibility(false, true, vec![]));
                        }
                        Token::Struct => structs.push(self.parse_struct(true)),
                        Token::Enum => enums.push(self.parse_enum(true)),
                        _ => { self.advance(); }
                    }
                }

                Token::At => {
                    let (attr_name, library_opt) = self.parse_attribute();
                    
                    if attr_name == "ffi" && self.current() == Token::Extern {
                        self.advance();
                        
                        let abi = if let Token::String(abi) = self.current() {
                            self.advance();
                            abi
                        } else {
                            "C".to_string()
                        };
                        
                        let library = library_opt.unwrap_or_else(|| "".to_string());
                        
                        self.expect(Token::Colon, vec![Token::End, Token::Func]);
                        
                        let mut functions_list = Vec::new();
                        
                        while !matches!(self.current(), Token::End | Token::EOF) {
                            if self.current() == Token::Func {
                                self.advance();
                                
                                let name = if let Token::Identifier(name) = self.current() {
                                    self.advance();
                                    name
                                } else {
                                    self.advance();
                                    "error".to_string()
                                };
                                
                                self.expect(Token::LeftParen, vec![Token::RightParen, Token::Colon]);
                                
                                let mut params = Vec::new();
                                while self.current() != Token::RightParen && self.current() != Token::EOF {
                                    if self.current() == Token::TripleDot {
                                        self.advance();
                                        params.push(("...".to_string(), Type::TripleDot));
                                        if self.current() == Token::Comma {
                                            self.advance();
                                        }
                                        continue;
                                    }
                                    
                                    let pname = if let Token::Identifier(pname) = self.current() {
                                        self.advance();
                                        pname
                                    } else {
                                        self.advance();
                                        "error".to_string()
                                    };
                                    
                                    self.expect(Token::Colon, vec![Token::Comma, Token::RightParen]);
                                    let ptype = self.parse_type();
                                    params.push((pname, ptype));
                                    
                                    if self.current() == Token::Comma {
                                        self.advance();
                                    }
                                }
                                
                                self.expect(Token::RightParen, vec![Token::Colon]);
                                
                                let return_type = if self.current() == Token::Arrow || self.current() == Token::Colon {
                                    self.advance();
                                    if self.is_type_token(self.current()) {
                                        self.parse_type()
                                    } else {
                                        Type::Void
                                    }
                                } else {
                                    Type::Void
                                };
                                
                                functions_list.push(ExternFunction {
                                    name,
                                    params,
                                    return_type,
                                    is_public: false,
                                });
                            } else if self.current() == Token::End {
                                break;
                            } else {
                                self.advance();
                            }
                            
                            if self.current() == Token::Semicolon {
                                self.advance();
                            }
                        }
                        
                        if self.current() == Token::End {
                            self.advance();
                        }
                        
                        externs.push(ExternDecl::Block { 
                            abi, 
                            library, 
                            functions: functions_list 
                        });
                    } else {
                         
                        self.diags.push(ParseDiagnostic {
                            message: format!("Unknown attribute: @{}", attr_name),
                            span: self.current_span(),
                            severity: DiagnosticSeverity::Error,
                            help: Some("Only @ffi is supported at the top level".to_string()),
                        });
                        self.advance();
                    }
                }

                Token::Mod => {
                     println!("DEBUG: Parsing Mod");
                     if let Token::String(_) = self.peek(1) {
                         self.advance();
                         if let Token::String(path) = self.current() {
                             self.advance();
                             imports.push(ModuleImport { path });
                         }
                     } else {
                         modules.push(self.parse_module(false));
                     }
                }

                    Token::Const => {
                        self.advance();
                        if let Token::Identifier(name) = self.current() {
                            self.advance();
                            let ty = if self.current() == Token::Colon {
                                self.advance();
                                self.parse_type()
                            } else {
                                Type::Void
                            };
                            
                            self.expect(Token::Equals, vec![Token::Semicolon]);
                            let value = self.parse_expr();
                            if self.current() == Token::Semicolon {
                                self.advance();
                            }
                            constants.push(GlobalConst { name, ty, value });
                        } else {
                            self.advance();
                        }
                    }

                Token::Type => {
                    println!("DEBUG: Parsing type alias");
                    let alias_stmt = self.parse_type_alias();
                     
                     
                    if self.current() == Token::Semicolon {
                        self.advance();
                    }
                }

                Token::Enum => {
                    enums.push(self.parse_enum(false));
                }

                Token::Func => {
                    println!("DEBUG: Parsing function");
                    let func = self.parse_function_with_visibility(false, false, vec![]);
                    functions.push(func);
                    println!("DEBUG: Functions count: {}", functions.len());
                }

                Token::Struct => {
                    println!("DEBUG: Parsing struct");
                    structs.push(self.parse_struct(false));
                }

                Token::Impl => {
                    self.advance();
                    
                    let struct_name = if let Token::Identifier(name) = self.current() {
                        self.advance();
                        name
                    } else {
                        self.advance();
                        "error".to_string()
                    };
                    
                    let mut constructor_params = Vec::new();
                    if self.current() == Token::LeftParen {
                        self.advance();
                        
                        while self.current() != Token::RightParen && self.current() != Token::EOF {
                            let pname = if let Token::Identifier(pname) = self.current() {
                                self.advance();
                                pname
                            } else {
                                self.advance();
                                "error".to_string()
                            };
                            
                            self.expect(Token::Colon, vec![Token::Comma, Token::RightParen]);
                            let ptype = self.parse_type();
                            constructor_params.push((pname, ptype));
                            
                            if self.current() == Token::Comma {
                                self.advance();
                            }
                        }
                        
                        self.expect(Token::RightParen, vec![Token::For, Token::Colon]);
                    }
                    
                    let trait_name = if self.current() == Token::For {
                        self.advance();
                        if let Token::Identifier(trait_name) = self.current() {
                            self.advance();
                            Some(trait_name)
                        } else {
                            self.advance();
                            None
                        }
                    } else {
                        None
                    };
                    
                    self.expect(Token::Colon, vec![Token::End]);
                    
                    let mut methods = Vec::new();
                    let mut constructor_body = None;
     
                    while !matches!(self.current(), Token::End | Token::EOF) {
                        let mut init_fields = Vec::new();        
                   
                        if let Token::Identifier(name) = self.current()
                            && name == struct_name && self.peek(1) == Token::LeftParen {
                                 
                                self.advance();
                                self.advance();
                                

                                while !matches!(self.current(), Token::RightParen | Token::EOF) {
                                    if let Token::Identifier(field_name) = self.current() {
                                        self.advance();
                                        self.expect(Token::Equals, vec![Token::Comma, Token::RightParen]);
                                        let field_expr = self.parse_expr();
                                        init_fields.push((field_name, field_expr));
                                        
                                        if self.current() == Token::Comma {
                                            self.advance();
                                        }
                                    } else {
                                        self.advance();
                                        break;
                                    }
                                }
                                
                                self.expect(Token::RightParen, vec![Token::Func, Token::End]);
                                constructor_body = Some(init_fields);
                                continue;
                            }
                        
                         
                        if self.current() == Token::Unsafe {
                            self.advance();
                        }
                        
                        let is_public = if self.current() == Token::Pub {
                            self.advance();
                            true
                        } else {
                            false
                        };

                        if self.current() == Token::Func {
                            self.advance();
                            
                            let method_name = if let Token::Identifier(name) = self.current() {
                                self.advance();
                                name
                            } else {
                                self.advance();
                                "error".to_string()
                            };
                            
                            self.expect(Token::LeftParen, vec![Token::RightParen, Token::Colon]);
                            
                            let mut params = Vec::new();
                            let mut self_modifier = None;
                            
                             
                            let is_self_token = |t: &Token| matches!(t, Token::Selfish) || matches!(t, Token::Identifier(s) if s == "self");
                            
                            if is_self_token(&self.current()) {
                                self.advance();
                                self_modifier = Some(SelfModifier::Immutable);
                                if self.current() == Token::Comma { self.advance(); }
                            } 
                            else if (matches!(self.current(), Token::Mutable | Token::Mut)) && is_self_token(&self.peek(1)) {
                                self.advance(); self.advance();
                                self_modifier = Some(SelfModifier::Mutable);
                                if self.current() == Token::Comma { self.advance(); }
                            }
                            else if (matches!(self.current(), Token::Reference | Token::Ampersand)) && is_self_token(&self.peek(1)) {
                                self.advance(); 
                                self.advance();

                                self_modifier = Some(SelfModifier::Reference);
                                if self.current() == Token::Comma { self.advance(); }
                            }
                            else if (matches!(self.current(), Token::Reference | Token::Ampersand)) && (matches!(self.peek(1), Token::Mutable | Token::Mut)) && is_self_token(&self.peek(2)) {
                                self.advance(); self.advance(); self.advance();
                                 
                                self_modifier = Some(SelfModifier::Mutable); 
                                if self.current() == Token::Comma { self.advance(); }
                            }
                            else if let Token::Identifier(s) = self.current()
                                && s == "brw" && is_self_token(&self.peek(1)) {
                                    self.advance(); self.advance();
                                    self_modifier = Some(SelfModifier::Borrow);
                                    if self.current() == Token::Comma { self.advance(); }
                                }
                                    
                            while self.current() != Token::RightParen && self.current() != Token::EOF {
                                let modifier = match self.current() {
                                    Token::Mutable => {
                                        self.advance();
                                        ParamModifier::Mutable
                                    }
                                    Token::Reference => {
                                        self.advance();
                                        ParamModifier::Reference
                                    }
                                    _ => ParamModifier::Immutable
                                };
                                
                                let pname = if let Token::Identifier(pname) = self.current() {
                                    self.advance();
                                    pname
                                } else {
                                    self.advance();
                                    "error".to_string()
                                };
                                
                                self.expect(Token::Colon, vec![Token::Comma, Token::RightParen]);
                                let ptype = self.parse_type();
                                params.push((pname, ptype, modifier));
                                
                                if self.current() == Token::Comma {
                                    self.advance();
                                }
                            }
                            
                            self.expect(Token::RightParen, vec![Token::Colon]);
                            
                            
                            let mut body = Vec::new();
                            let return_type = if self.current() == Token::Arrow || self.current() == Token::Colon {
                                self.advance();
                                if self.is_type_token(self.current()) {
                                    self.parse_type()
                                } else {
                                    Type::Void
                                }
                            } else {
                                Type::Void
                            };
                            
                            while !matches!(self.current(), Token::End | Token::Func | Token::EOF) {
                            
                                if matches!(self.current(), Token::End | Token::Func) {
                                    break;
                                }
                                
                                body.push(self.parse_stmt());
                                if self.current() == Token::Semicolon {
                                    self.advance();
                                }
                            }
                            
                            if self.current() == Token::End {
                                self.advance();
                            }
                            
                            methods.push(ImplMethod {
                                name: method_name,
                                params,
                                return_type,
                                body,
                                self_modifier,
                                is_public,
                            });
                        } else if self.current() == Token::End {
                            break;
                        } else {
                            self.advance();
                        }
                    }
                    
                    if self.current() == Token::End {
                        self.advance();
                    }
                    
                    impls.push(ImplBlock {
                        struct_name,
                        trait_name,
                        constructor_params,
                        constructor_body,
                        methods,
                    });
                }

                Token::Use => {
                    self.advance();
                    if let Token::Identifier(module_name) = self.current() {
                        self.advance();
                        uses.push(ModuleUse { module_name });
                    } else {
                        self.advance();
                    }
                }

                Token::Extern => {
                    self.advance();
                    
                    let abi = if let Token::String(abi) = self.current() {
                        self.advance();
                        abi
                    } else {
                        "C".to_string()
                    };
                    
                    
                    if self.current() == Token::Colon {
                        self.advance();
                        
                        let mut functions_list = Vec::new();
                        
                        while !matches!(self.current(), Token::End | Token::EOF) {
                            if self.current() == Token::Func {
                                self.advance();
                                
                                let name = if let Token::Identifier(name) = self.current() {
                                    self.advance();
                                    name
                                } else {
                                    self.advance();
                                    "error".to_string()
                                };
                                
                                self.expect(Token::LeftParen, vec![Token::RightParen, Token::Colon]);
                                
                                let mut params = Vec::new();
                                while self.current() != Token::RightParen && self.current() != Token::EOF {
                                    if self.current() == Token::TripleDot {
                                        self.advance();
                                        params.push(("...".to_string(), Type::TripleDot));
                                        if self.current() == Token::Comma {
                                            self.advance();
                                        }
                                        continue;
                                    }
                                    
                                    let pname = if let Token::Identifier(pname) = self.current() {
                                        self.advance();
                                        pname
                                    } else {
                                        self.advance();
                                        "error".to_string()
                                    };
                                    
                                    self.expect(Token::Colon, vec![Token::Comma, Token::RightParen]);
                                    let ptype = self.parse_type();
                                    params.push((pname, ptype));
                                    
                                    if self.current() == Token::Comma {
                                        self.advance();
                                    }
                                }
                                
                                self.expect(Token::RightParen, vec![Token::Colon, Token::Arrow]);
                                
                                let return_type = if self.current() == Token::Arrow || self.current() == Token::Colon {
                                    self.advance();
                                    if self.is_type_token(self.current()) {
                                        self.parse_type()
                                    } else {
                                        Type::Void
                                    }
                                } else {
                                    Type::Void
                                };
                                
                                functions_list.push(ExternFunction {
                                    name,
                                    params,
                                    return_type,
                                    is_public: false,
                                });
                            } else if self.current() == Token::End {
                                break;
                            } else {
                                self.advance();
                            }
                            
                            if self.current() == Token::Semicolon {
                                self.advance();
                            }
                        }
                        
                        if self.current() == Token::End {
                            self.advance();
                        }
                        
                        externs.push(ExternDecl::Block { 
                            abi, 
                            library: String::new(), 
                            functions: functions_list 
                        });
                    } else {
                        
                        let name = if let Token::Identifier(name) = self.current() {
                            self.advance();
                            name
                        } else {
                            self.advance();
                            "error".to_string()
                        };
                        
                        self.expect(Token::LeftParen, vec![Token::RightParen]);
                        
                        let mut params = Vec::new();
                        while self.current() != Token::RightParen && self.current() != Token::EOF {
                            if self.current() == Token::TripleDot {
                                self.advance();
                                params.push(("...".to_string(), Type::TripleDot));
                                if self.current() == Token::Comma {
                                    self.advance();
                                }
                                continue;
                            }
                            
                            let pname = if let Token::Identifier(pname) = self.current() {
                                self.advance();
                                pname
                            } else {
                                self.advance();
                                "error".to_string()
                            };
                            
                            self.expect(Token::Colon, vec![Token::Comma, Token::RightParen]);
                            let ptype = self.parse_type();
                            params.push((pname, ptype));
                            
                            if self.current() == Token::Comma {
                                self.advance();
                            }
                        }
                        
                        self.expect(Token::RightParen, vec![Token::Colon, Token::Arrow, Token::Semicolon]);
                        
                        let return_type = if self.current() == Token::Arrow || self.current() == Token::Colon {
                            self.advance();
                            if self.is_type_token(self.current()) {
                                self.parse_type()
                            } else {
                                Type::Void
                            }
                        } else {
                            Type::Void
                        };
                        
                        externs.push(ExternDecl::Single {
                            abi,
                            func: ExternFunction {
                                name,
                                params,
                                return_type,
                                is_public: false,
                            },
                        });
                    
                        if self.current() == Token::Semicolon {
                            self.advance();
                        }
                    }
                }

                Token::Hash => {
                    let mut attributes = Vec::new();
                    while self.current() == Token::Hash {
                        let (name, param) = self.parse_attribute();
                        attributes.push(if let Some(p) = param {
                            format!("{}({})", name, p)
                        } else {
                            name
                        });
                    }
                    
                    let is_public = if self.current() == Token::Pub {
                        self.advance();
                        true
                    } else {
                        false 
                    };
                    
                    if self.current() == Token::Func {
                        let func = self.parse_function_with_visibility(false, is_public, attributes);
                        functions.push(func);
                    } else {
                        self.diags.push(ParseDiagnostic {
                            message: "Expected function after attributes".to_string(),
                            span: self.current_span(),
                            severity: DiagnosticSeverity::Error,
                            help: Some("Attributes are currently only supported on functions".to_string()),
                        });
                        self.advance();
                    }
                }

                Token::Module => {
                    self.advance();
                    functions.push(self.parse_function(true));
                }
                
                
                Token::Type => {
                    self.advance();
                    let _name = if let Token::Identifier(name) = self.current() { self.advance(); name } else { self.advance(); "error".to_string() };
                    self.expect(Token::Equals, vec![Token::End]);
                    let _ty = self.parse_type();
                }

                Token::Trait => {
                    self.advance();
                    let _name = if let Token::Identifier(name) = self.current() { self.advance(); name } else { self.advance(); "error".to_string() };
                    self.expect(Token::Colon, vec![Token::End]);
                    while !matches!(self.current(), Token::End | Token::EOF) {
                        self.advance(); 
                    }
                    if self.current() == Token::End { self.advance(); }
                }

                Token::Identifier(ref id) if id == "export" => {
                    println!("DEBUG: Parsing module exports");
                    let export_stmt = self.parse_module_exports();
                     
                    modules.push(export_stmt);
                    if self.current() == Token::Semicolon {
                        self.advance();
                    }
                }

                _ => {
                    self.advance();
                }
            }
        }
        
        let undefined = self.find_undefined_functions(&functions, &externs, &import_context);
            
        (Program { functions, constants, modules }, structs, enums, externs, imports, uses, classes, impls, traits, undefined, import_decls)
    }
    
    fn find_undefined_functions(
        &self, 
        functions: &[Function], 
        externs: &[ExternDecl],
        import_context: &ImportContext
    ) -> UndefinedFunctions {
        let mut library_functions = Vec::new();
        let mut defined_functions = HashSet::new();
        
         
        for func in functions {
            defined_functions.insert(func.name.clone());
        }
        
         
        for ext in externs {
            match ext {
                ExternDecl::Single { func, .. } => {
                    defined_functions.insert(func.name.clone());
                }
                ExternDecl::SingleWithBody { func, .. } => {
                    defined_functions.insert(func.name.clone());
                }
                ExternDecl::Block { functions, .. } => {
                    for f in functions {
                        defined_functions.insert(f.name.clone());
                    }
                }
            }
        }
    
         
        for func in functions {
            self.stmt_calls(&func.body, &defined_functions, &mut library_functions, import_context);
        }
        
        UndefinedFunctions { library_functions }
    }
    
    fn stmt_calls(
        &self,
        stmts: &[Stmt],
        defined: &HashSet<String>,
        undefined: &mut Vec<UndefinedFunction>,
        import_context: &ImportContext
    ) {
        for stmt in stmts {
            match stmt {
                Stmt::Call(name, args) => {
                     
                    if import_context.is_library_function(name) {
                         
                        continue;
                    }
                    
                    if !defined.contains(name)
                        && !undefined.iter().any(|u| u.name == *name) {
                            undefined.push(UndefinedFunction {
                                name: name.clone(),
                                call_location: SourceSpan::from(0..0), 
                                args_count: args.len(),
                            });
                        }
                    
                    for arg in args {
                        self.expr_calls(arg, defined, undefined, import_context);
                    }
                }
                Stmt::ModuleCall(module, _func, args) => {
                     
                    if import_context.is_imported_symbol(module) {
                         
                        continue;
                    }
                    
                    for arg in args {
                        self.expr_calls(arg, defined, undefined, import_context);
                    }
                }
                Stmt::If(cond, then_body, else_body) => {
                    self.expr_calls(cond, defined, undefined, import_context);
                    self.stmt_calls(then_body, defined, undefined, import_context);
                    if let Some(else_stmts) = else_body {
                        self.stmt_calls(else_stmts, defined, undefined, import_context);
                    }
                }
                Stmt::While(cond, body) => {
                    self.expr_calls(cond, defined, undefined, import_context);
                    self.stmt_calls(body, defined, undefined, import_context);
                }
                Stmt::Match(expr, cases, default) => {
                    self.expr_calls(expr, defined, undefined, import_context);
                    for case in cases {
                        self.expr_calls(&case.value, defined, undefined, import_context);
                        self.stmt_calls(&case.body, defined, undefined, import_context);
                    }
                    if let Some(default_body) = default {
                        self.stmt_calls(default_body, defined, undefined, import_context);
                    }
                }
                Stmt::Scope(body) | Stmt::Unsafe(body) => {
                    self.stmt_calls(body, defined, undefined, import_context);
                }
                Stmt::Return(Some(expr)) | Stmt::Assign(_, expr) | Stmt::CompoundAssign(_, _, expr) => {
                    self.expr_calls(expr, defined, undefined, import_context);
                }
                _ => {}
            }
        }
    }
    
    fn expr_calls(
        &self,
        expr: &Expr,
        defined: &HashSet<String>,
        undefined: &mut Vec<UndefinedFunction>,
        import_context: &ImportContext
    ) {
        match expr {
            Expr::Call(name, args) => {
                if import_context.is_library_function(name) {
                     
                    return;
                }
                
                if !defined.contains(name)
                    && !undefined.iter().any(|u| u.name == *name) {
                        undefined.push(UndefinedFunction {
                            name: name.clone(),
                            call_location: SourceSpan::from(0..0),
                            args_count: args.len(),
                        });
                    }
                for arg in args {
                    self.expr_calls(arg, defined, undefined, import_context);
                }
            }
            Expr::ModuleCall(module, _func, args) => {
                if import_context.is_imported_symbol(module) {
                     
                    return;
                }
                
                for arg in args {
                    self.expr_calls(arg, defined, undefined, import_context);
                }
            }
            Expr::BinOp(_, left, right) => {
                self.expr_calls(left, defined, undefined, import_context);
                self.expr_calls(right, defined, undefined, import_context);
            }
            Expr::UnOp(_, expr) => {
                self.expr_calls(expr, defined, undefined, import_context);
            }
            Expr::Tuple(exprs) | Expr::Array(exprs) => {
                for e in exprs {
                    self.expr_calls(e, defined, undefined, import_context);
                }
            }
            _ => {}
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Token::lexer::Lexer;

    fn parse_func(code: &str) -> Result<Function, String> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        if !lexer.errors.is_empty() {
             return Err(format!("Lexer errors: {:?}", lexer.errors));
        }
        let spans = lexer.spans.clone();
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut parser = Parser::new(tokens, code.to_string(), spans);
            let func = parser.parse_function(false);
            if !parser.diags.is_empty() {
                Err(format!("Parser diagnostics: {:?}", parser.diags))
            } else {
                Ok(func)
            }
        }));

        match result {
            Ok(Ok(func)) => Ok(func),
            Ok(Err(msg)) => Err(msg),
            Err(e) => Err(format!("Parser panicked: {:?}", e)),
        }
    }

    #[test]
    fn test_func_parsing() {
       let result = parse_func("func main()");
       assert!(result.is_err(), "Expected parsing error due to missing 'end' keyword");
    }

    #[test]
    fn test_attribute_parsing() {
        let code = "#[Test] func test_attr() end";
        let mut lexer = Lexer::new(code);
        let spans = lexer.spans.clone();
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens, code.to_string(), spans);

        let func = parser.parse_function_with_visibility(false, false, vec!["Test".to_string()]);
        assert_eq!(func.name, "test_attr");
        assert!(func.attributes.contains(&"Test".to_string()));
    }

    #[test]
    fn test_full_game_parser() {
        let game_render_code = r#"
extern "stdcall" from "user32.dll":
    func ShowWindow(hWnd: ptr[void], nCmdShow: int32): int32
end

const SW_SHOW: int32 = 5

func RGB(r: uint8, g: uint8, b: uint8): uint32
    return (r as uint32)
end

struct PlayerData:
    mut names: str[]
    mut count: int
end

func WindowProc(hwnd: ptr[void], msg: uint32): int
    if msg == 2 then
        return 0
    end
    return 1
end

public func add_player(name: str): int
    return 1
end
"#;

        let main_code = r#"
import game_render

struct Game:
    mut current_players = int
end

impl Game:
    func start(): Game
        Game(current_players = 0)
    end

    func add_player(&mut self, name: str, health: int): Result[(), str]
        return Ok()
    end
end

func main()
    let mut game = Game.start()
end
"#;

        let mut lexer1 = Lexer::new(game_render_code);
        let tokens1 = lexer1.tokenize();
        let spans1 = lexer1.spans.clone();
        let parser1 = Parser::new(tokens1, game_render_code.to_string(), spans1);
        let (program1, structs1, _enums1, externs1, _imports1, _uses1, _classes1, _impls1, _traits1, _undef1, _import_decls1) = parser1.parse();

        println!("Parsed: {} functions, {} structs, {} externs, {} constants", program1.functions.len(), structs1.len(), externs1.len(), program1.constants.len());

        assert!(true, "Parser completed successfully");

        let mut lexer2 = Lexer::new(main_code);
        let tokens2 = lexer2.tokenize();
        let spans2 = lexer2.spans.clone();
        let parser2 = Parser::new(tokens2, main_code.to_string(), spans2);
        let (program2, structs2, _enums2, _externs2, _imports2, _uses2, _classes2, impls2, _traits2, _undef2, _import_decls2) = parser2.parse();
        
        println!("Main parsed: {} functions, {} structs, {} impls", program2.functions.len(), structs2.len(), impls2.len());
        assert!(true, "Parser completed successfully on main code");
    }
}




