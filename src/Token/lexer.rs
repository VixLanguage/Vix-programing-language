use crate::import::*;

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: SourceSpan,
}

pub struct Lexer {
    source: String,
    chars: Vec<char>,
    pos: usize,
    pub errors: Vec<LexError>,
    pub spans: Vec<SourceSpan>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.to_string(),
            chars: source.chars().collect(),
            pos: 0,
            errors: Vec::new(),
            spans: Vec::new(),
        }
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.current() == Some('/') && self.peek(1) == Some('/') {
            while self.current().is_some() && self.current() != Some('\n') {
                self.advance();
            }
        }
    }

    fn read_char(&mut self) -> Token {
        self.advance();  
        
        let ch = if self.current() == Some('\\') {
            self.advance();  
            match self.current() {
                Some('n') => { self.advance(); '\n' }
                Some('t') => { self.advance(); '\t' }
                Some('r') => { self.advance(); '\r' }
                Some('\\') => { self.advance(); '\\' }
                Some('\'') => { self.advance(); '\'' }
                Some('0') => { self.advance(); '\0' }
                Some('x') => {
                    self.advance();
                    let mut hex = String::new();
                    for _ in 0..2 {
                        if let Some(h) = self.current() {
                            if h.is_ascii_hexdigit() {
                                hex.push(h);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    if let Ok(val) = u8::from_str_radix(&hex, 16) {
                        val as char
                    } else {
                        '?'  
                    }
                }
                _ => {
                    let c = self.current().unwrap_or('?');
                    self.advance();
                    c
                }
            }
        } else {
            let c = self.current().unwrap_or('?');
            self.advance();
            c
        };
        
        
        if self.current() == Some('\'') {
            self.advance();
        } else {
            self.errors.push(LexError {
                message: "Unclosed character literal".to_string(),
                span: SourceSpan::from(self.pos..self.pos + 1),
            });
        }
        
        Token::Char(ch as i32)
    }

    fn read_number(&mut self) -> Token {
        let _start = self.pos;

 
        if self.current() == Some('0') {
            if self.peek(1) == Some('x') || self.peek(1) == Some('X') {
                self.advance(); 
                self.advance(); 
                let mut num_str = String::new();
                while let Some(ch) = self.current() {
                    if ch.is_ascii_hexdigit() {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let num = u32::from_str_radix(&num_str, 16).unwrap_or(0);
                return Token::HexNumber(num);
            } else if self.peek(1) == Some('b') || self.peek(1) == Some('B') {
                self.advance(); 
                self.advance(); 
                let mut num_str = String::new();
                while let Some(ch) = self.current() {
                    if ch == '0' || ch == '1' {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let num = u32::from_str_radix(&num_str, 2).unwrap_or(0);
                return Token::BinaryNumber(num);
            } else if self.peek(1) == Some('o') || self.peek(1) == Some('O') {
                self.advance(); 
                self.advance(); 
                let mut num_str = String::new();
                while let Some(ch) = self.current() {
                    if ch.is_digit(8) {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let num = u32::from_str_radix(&num_str, 8).unwrap_or(0);
                return Token::OctalNumber(num);
            }
        }

 
        let mut num_str = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

 
        if self.current() == Some('.') && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
            num_str.push('.');
            self.advance();
            while let Some(ch) = self.current() {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            let float_val = num_str.parse::<f32>().unwrap_or(0.0);
            return Token::Float(OrderedFloat(float_val));
        }

        let num = num_str.parse::<i64>().unwrap_or(0);
        Token::Number(num)
    }

       fn read_string(&mut self) -> Token {
        self.advance();
        let mut string = String::new();

        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance();
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current() {
                    match escaped {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        'x' => {
                             
                            self.advance();
                            let mut hex = String::new();
                            
                             
                            for _ in 0..2 {
                                if let Some(h) = self.current() {
                                    if h.is_ascii_hexdigit() {
                                        hex.push(h);
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            
                            if !hex.is_empty() {
                                if let Ok(val) = u8::from_str_radix(&hex, 16) {
                                    string.push(val as char);
                                } else {
                                     
                                    string.push('\\');
                                    string.push('x');
                                    string.push_str(&hex);
                                }
                            } else {
                                 
                                string.push('\\');
                                string.push('x');
                            }
                            continue;  
                        }
                        '0' => string.push('\0'),
                        _ => {
                             
                            string.push('\\');
                            string.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {               
                string.push(ch);
                self.advance();
            }
        }

        Token::String(string)
    }
    
    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

 
        let is_type_ident = ident.chars().next().is_some_and(|c| c.is_lowercase()) &&
                           (ident.starts_with("int") || 
                            ident.starts_with("uint") || 
                            ident.starts_with("float")
                        );

        if is_type_ident {
            return Token::TypeIdentifier(ident);
        }

 
        match ident.as_str() {
            "create" => Token::Let,
            "public" => Token::Pub,
            "func" => Token::Func,
            "enum" => Token::Enum,
            "any" => Token::Any,
            "end" => Token::End,
            "as" => Token::As,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "module" => Token::Module,
            "true" => Token::True,
            "false" => Token::False,
            "extern" => Token::Extern,
            "from" => Token::From,
            "mut" => Token::Mut,
            "struct" => Token::Struct,
            "mod" => Token::Mod,
            "to" => Token::To,
            "use" => Token::Use,
            "match" => Token::Match,
            "case" => Token::Case,
            "default" => Token::Default,
            "unsafe" => Token::Unsafe,
            "then" => Token::Then,
            "do" => Token::Do,
            "in" => Token::In,
            "scope" => Token::Scope,
            "plan" => Token::Plan,
            "const" => Token::Const,
            "mutable" => Token::Mutable,
            "reference" => Token::Reference,
            "None" => Token::None,
            "null" => Token::Null,
            "impl" => Token::Impl,
            "self" => Token::Selfish,
            "trait" => Token::Trait,
            "import" => Token::Import,
            "array" => Token::Array,
            "slots" => Token::Slots,
            "lists" => Token::Lists,
            "panic" => Token::Panic,
            "have" => Token::Have,
            "get" => Token::Get,
            "filter" => Token::Filter,
            "is_empty" => Token::IsEmpty,
            "unwrap" => Token::Unwrap,
            "unwrap_or" => Token::UnwrapOr,
            "is_some" => Token::IsSome,
            "is_none" => Token::IsNone,
            "ok" | "Ok" => Token::Ok,
            "err" | "Err" => Token::Err,
            "not" => Token::Not,
            "or" => Token::Or,
            "reference_to" => Token::ReferenceTo,
            "some" | "Some" => Token::Some,
            "result" | "Result" => Token::Result,
            "option" | "Option" => Token::Option,
            "wait" => Token::Wait,
            "nullptr" => Token::NullPtr,
            "sizeof" => Token::SizeOf,
            "typeof" => Token::TypeOf,
            "alignof" => Token::AlignOf,
            "offsetof" => Token::OffsetOf,
            "oneof" => Token::OneOf,
            "bool" => Token::Bool,
            "void" => Token::Void,
            "str" => Token::Str,
            "String" => Token::StdStr,
            "type" => Token::Type,
            ".." => Token::DoubleDot,
 
            "int8" => Token::TypeIdentifier("int8".to_string()),
            "int16" => Token::TypeIdentifier("int16".to_string()),
            "int32" => Token::TypeIdentifier("int32".to_string()),
            "int64" => Token::TypeIdentifier("int64".to_string()),
            "uint8" => Token::TypeIdentifier("uint8".to_string()),
            "uint16" => Token::TypeIdentifier("uint16".to_string()),
            "uint32" => Token::TypeIdentifier("uint32".to_string()),
            "uint64" => Token::TypeIdentifier("uint64".to_string()),
            "float32" => Token::TypeIdentifier("float32".to_string()),
            "float64" => Token::TypeIdentifier("float64".to_string()),
            "string" => Token::TypeIdentifier("string".to_string()),
            
            _ => Token::Identifier(ident),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.pos < self.chars.len() {
            let start = self.pos;
            self.skip_whitespace();

            if self.pos >= self.chars.len() {
                break;
            }

 
            if self.current() == Some('/') && self.peek(1) == Some('/') {
                self.skip_comment();
                continue;
            }

            let token = match self.current() {
                Some('(') => { self.advance(); Token::LeftParen }
                Some(')') => { self.advance(); Token::RightParen }
                Some('{') => { self.advance(); Token::LeftBrace }
                Some('}') => { self.advance(); Token::RightBrace }
                Some('[') => { self.advance(); Token::LeftBracket }
                Some(']') => { self.advance(); Token::RightBracket }
                Some(';') => { self.advance(); Token::Semicolon }
                Some(',') => { self.advance(); Token::Comma }
                Some('^') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::CaretEquals
                    } else {
                        Token::Caret
                    }
                }
                Some(':') => {
                    self.advance();
                    if self.current() == Some(':') {
                        self.advance();
                        Token::DoubleColon
                    } else {
                        Token::Colon
                    }
                }
                Some('.') => {
                    self.advance();
                    if self.current() == Some('.') && self.peek(1) == Some('.') {
                        self.advance();
                        self.advance();
                        Token::TripleDot
                    } else {
                        Token::Dot
                    }
                }
                Some('+') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::PlusEquals
                    } else {
                        Token::Plus
                    }
                }
                Some('-') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::MinusEquals
                    } else if self.current() == Some('>') {
                        self.advance();
                        Token::Arrow
                    } else {
                        Token::Minus
                    }
                }
                Some('*') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::StarEquals
                    } else {
                        Token::Star
                    }
                }
                Some('/') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::SlashEquals
                    } else {
                        Token::Slash
                    }
                }
                Some('%') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::PercentEquals
                    } else {
                        Token::Percent
                    }
                }
                Some('=') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::EqualsEquals
                    } else {
                        Token::Equals
                    }
                }
                Some('!') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::NotEquals
                    } else {
                        Token::Not
                    }
                }
                Some('<') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::LessEquals
                    } else if self.current() == Some('<') {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            Token::LessLessEquals
                        } else {
                            Token::LeftShift
                        }
                    } else {
                        Token::Less
                    }
                }
                Some('>') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::GreaterEquals
                    } else if self.current() == Some('>') {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            Token::GreaterGreaterEquals
                        } else {
                            Token::RightShift
                        }
                    } else {
                        Token::Greater
                    }
                }
                Some('&') => {
                    self.advance();
                    if self.current() == Some('&') {
                        self.advance();
                        Token::And
                    } else if self.current() == Some('=') {
                        self.advance();
                        Token::AmpersandEquals
                    } else {
                        Token::Ampersand
                    }
                }
                Some('|') => {
                    self.advance();
                    if self.current() == Some('|') {
                        self.advance();
                        Token::Or
                    } else if self.current() == Some('=') {
                        self.advance();
                        Token::PipeEquals
                    } else {
                        Token::Pipe
                    }
                }
                Some('~') => { self.advance(); Token::Tilde }
                Some('"') => self.read_string(),
                Some('\'') => self.read_char(),
                Some(ch) if ch.is_ascii_digit() => self.read_number(),
                Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
                Some(ch) => {
                    self.errors.push(LexError {
                        message: format!("Unexpected character: '{}'", ch),
                        span: SourceSpan::from(start..self.pos + 1),
                    });
                    self.advance();
                    continue;
                }
                None => break,
            };

            let end = self.pos;
            self.spans.push(SourceSpan::from(start..end));
            tokens.push(token);
        }

        tokens.push(Token::EOF);
        self.spans.push(SourceSpan::from(self.pos..self.pos));
        
        tokens
    }
}