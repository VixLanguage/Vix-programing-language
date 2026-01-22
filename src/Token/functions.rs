use crate::import::*;

impl Parser {
    pub fn parse_type_identifier(type_name: &str) -> Type {
        if let Some(bits_str) = type_name.strip_prefix("int")
            && let Ok(bits) = bits_str.parse::<usize>() {
                return Type::int(bits, true);
            }

        if type_name == "char" {
            return Type::char32();
        }

        if let Some(bits_str) = type_name.strip_prefix("uint")
            && let Ok(bits) = bits_str.parse::<usize>() {
                return Type::int(bits, false);
            }

        if let Some(bits_str) = type_name.strip_prefix("float")
            && let Ok(bits) = bits_str.parse::<usize>() {
                return Type::float(bits);
            }

        Type::i32()
    }

    pub fn infer_type(expr: &Expr) -> Type {
        match expr {
            Expr::HashMap(entries) if !entries.is_empty() => {
                let key_ty = Self::infer_type(&entries[0].0);
                let val_ty = Self::infer_type(&entries[0].1);
                Type::HashMap {
                    key: Box::new(key_ty),
                    value: Box::new(val_ty),
                }
            }

            Expr::Number(_) => Type::i32(),
            Expr::HexNumber(_) => Type::i32(),
            Expr::BinaryNumber(_) => Type::i32(),
            Expr::OctalNumber(_) => Type::i32(),
            Expr::Float(_) => Type::f32(),
            Expr::String(_) => Type::Str { len_type: Box::new(Type::i32()) },
            Expr::Bool(_) => Type::Bool,
            Expr::Char(_) => Type::char32(),
            Expr::Some(inner) => {
                Type::Option { 
                    inner: Box::new(Parser::infer_type(inner)) 
                }
            }
            Expr::None => {
                Type::Option { 
                    inner: Box::new(Type::Any) 
                }
            }
            
            Expr::ResultOk(inner) => {
                Type::Result { 
                    ok: Box::new(Parser::infer_type(inner)), 
                    err: Box::new(Type::Any)
                }
            }
            Expr::ResultErr(inner) => {
                Type::Result { 
                    ok: Box::new(Type::Any),
                    err: Box::new(Parser::infer_type(inner)) 
                }
            }
            
            Expr::Unwrap(inner) => {
                let inner_type = Parser::infer_type(inner);
                match inner_type {
                    Type::Option { inner: boxed } => *boxed,
                    Type::Result { ok, .. } => *ok,
                    _ => inner_type,
                }
            }
            Expr::UnwrapOr(inner, default) => {
                let inner_type = Parser::infer_type(inner);
                let default_type = Parser::infer_type(default);
                match inner_type {
                    Type::Option { inner: boxed } => *boxed,
                    Type::Result { ok, .. } => *ok,
                    _ => default_type,
                }
            }
            
            Expr::OptionMethod { obj, method, .. } => {
                match method.as_str() {
                    "is_some" | "is_none" => Type::Bool,
                    _ => Parser::infer_type(obj),
                }
            }
            
            Expr::Cast(_, cast_target) => {
                match cast_target {
                    CastTarget::Type(ty) => ty.clone(),
                    CastTarget::LibraryCall(_, _) => Type::i32(),
                    CastTarget::LibraryCallTyped(_, types) => {
                        types.last().cloned().unwrap_or(Type::i32())
                    }
                    CastTarget::LibraryModuleCall(_, _, _) => Type::i32(),
                }
            }
            
            Expr::Tuple(elements) => {
                let types = elements.iter().map(Parser::infer_type).collect();
                Type::Tuple { fields: types }
            }
            
            Expr::Array(elements) => {
                if elements.is_empty() {
                    Type::Array {
                        element: Box::new(Type::i32()),
                        size: Some(0),
                    }
                } else {
                    let mut types = std::collections::HashSet::new();
                    for elem in elements {
                        types.insert(Parser::infer_type(elem));
                    }
                    let elem_type = if types.len() == 1 {
                        types.into_iter().next().unwrap()
                    } else {
                        Type::Union {
                            variants: types.into_iter().collect(),
                        }
                    };
                    Type::Array {
                        element: Box::new(elem_type),
                        size: Some(elements.len()),
                    }
                }
            }
            
            Expr::CallNamed(name, _) => Type::Struct { name: name.clone() },
            Expr::ModuleCallNamed(_, name, _) => Type::Struct { name: name.clone() },
            Expr::IsEmpty(_) | Expr::IsNotEmpty(_) => Type::Bool,
            Expr::Have { .. } | Expr::Contain { .. } | Expr::ContainAll { .. } => Type::Bool,
            Expr::Index(arr, _) => {
                let arr_type = Parser::infer_type(arr);
                match arr_type {
                    Type::Array { element, .. } => *element,
                    Type::Ptr(inner) => *inner,
                    _ => Type::i32(),
                }
            }

            Expr::IndexOf { .. } => Type::i32(),
            Expr::SizeOf(_) | Expr::AlignOf(_) => Type::u64(),
            Expr::TypeOf(_) => Type::Str { len_type: Box::new(Type::i32()) },
            Expr::Panic(_) => Type::Void,
            Expr::ReferenceTo(_) => Type::u32(),
            Expr::Collect(_) => Type::Ptr(Box::new(Type::Void)),
            Expr::Filter { obj, .. } => Parser::infer_type(obj),
            Expr::Wait(inner) => {
                let inner_type = Parser::infer_type(inner);
                Type::Ptr(Box::new(inner_type))
            }

            Expr::Chars(_) => Type::Array { 
                element: Box::new(Type::char8()), 
                size: None 
            },

            
            _ => Type::i32(),
        }
    }


    pub fn parse_expr_list(&mut self) -> Vec<Expr> {
        let mut exprs = Vec::new();
        while self.current() != Token::RightParen && self.current() != Token::Semicolon && self.current() != Token::EOF {
            exprs.push(self.parse_expr());
            if self.current() == Token::Comma {
                self.advance();
                    } else {
                break;
            }
        }
        exprs
    }

    pub fn parse_op(&mut self) -> Expr {
        self.parse_unary()
    }

    pub fn parse_compound_op(&self) -> Option<String> {
        match self.current() {
            Token::PlusEquals => Some("+=".to_string()),
            Token::MinusEquals => Some("-=".to_string()),
            Token::StarEquals => Some("*=".to_string()),
            Token::SlashEquals => Some("/=".to_string()),
            Token::PercentEquals => Some("%=".to_string()),
            Token::AmpersandEquals => Some("&=".to_string()),
            Token::PipeEquals => Some("|=".to_string()),
            Token::CaretEquals => Some("^=".to_string()),
            Token::LessLessEquals => Some("<<=".to_string()),
            Token::GreaterGreaterEquals => Some(">>=".to_string()),
            _ => None,
        }
    }


    pub fn parse_method_call(&mut self, obj: Expr, method_name: String) -> Expr {
        self.expect(Token::LeftParen, vec![Token::RightParen]);

        let mut args = Vec::new();
        if !matches!(self.current(), Token::RightParen | Token::EOF) {
            loop {
                args.push(self.parse_expr());
                if self.current() == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::RightParen, vec![Token::Semicolon, Token::End]);
        Expr::MethodCall(Box::new(obj), method_name, args)
    }
}