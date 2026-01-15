use crate::import::*;

impl Parser {
     pub fn current(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF)
    }

    pub fn peek(&self, n: usize) -> Token {
        self.tokens.get(self.pos + n).cloned().unwrap_or(Token::EOF)
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    fn current_span(&self) -> SourceSpan {
        self.spans.get(self.pos).cloned().unwrap_or(SourceSpan::from(0..0))
    }

    pub fn expect(&mut self, expected: Token, sync: Vec<Token>) {
        let _start_pos = self.pos;
        let current = self.current();
        if current == expected {
            self.advance();
            return;
        }
        let sync_set: std::collections::HashSet<Token> = sync.into_iter().collect();
        let mut skipped = 0;
        let max_skip = 50;
        while self.pos < self.tokens.len() && skipped < max_skip {
            let current_token = self.current();
            if current_token == expected {
                self.advance();
                return;
            }
            if sync_set.contains(&current_token) {
                return;
            }
            if current_token == Token::EOF {
                return;
            }
            self.advance();
            skipped += 1;
        }
    }
}