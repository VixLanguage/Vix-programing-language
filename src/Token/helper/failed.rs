
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