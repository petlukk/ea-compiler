// Updated parser primary() function with SIMD and array indexing support

/// Parses primary expressions: literals, variables, parentheses, SIMD vectors, and arrays
fn primary(&mut self) -> Result<Expr> {
    // Handle literals and variables
    if self.match_tokens(&[TokenKind::True]) {
        return Ok(Expr::Literal(Literal::Boolean(true)));
    }
    
    if self.match_tokens(&[TokenKind::False]) {
        return Ok(Expr::Literal(Literal::Boolean(false)));
    }
    
    if let Some(token) = self.match_tokens_and_get(&[TokenKind::Integer(0)]) {
        if let TokenKind::Integer(n) = token.kind {
            return Ok(Expr::Literal(Literal::Integer(n)));
        }
    }
    
    if let Some(token) = self.match_tokens_and_get(&[TokenKind::Float(0.0)]) {
        if let TokenKind::Float(f) = token.kind {
            return Ok(Expr::Literal(Literal::Float(f)));
        }
    }
    
    if let Some(token) = self.match_tokens_and_get(&[TokenKind::StringLiteral("".to_string())]) {
        if let TokenKind::StringLiteral(s) = token.kind {
            return Ok(Expr::Literal(Literal::String(s)));
        }
    }
    
    if let Some(token) = self.match_tokens_and_get(&[TokenKind::Identifier("".to_string())]) {
        if let TokenKind::Identifier(name) = token.kind {
            return Ok(Expr::Variable(name));
        }
    }

    // Handle SIMD literals that already come as a single token
    if let Some(token) = self.match_tokens_and_get(&[TokenKind::SimdLiteral("".to_string())]) {
        if let TokenKind::SimdLiteral(s) = token.kind {
            return Ok(Expr::Literal(Literal::String(s))); // Temporary - will be enhanced in SIMD type system
        }
    }

    // Handle grouping with parentheses
    if self.match_tokens(&[TokenKind::LeftParen]) {
        let expr = self.expression()?;
        self.consume(
            TokenKind::RightParen,
            "Expected ')' after expression".to_string(),
        )?;
        return Ok(Expr::Grouping(Box::new(expr)));
    }

    // Handle array literals and SIMD vector literals starting with [
    if self.match_tokens(&[TokenKind::LeftBracket]) {
        return self.parse_array_or_simd_literal();
    }

    // If we get here, we couldn't match any expression
    Err(CompileError::parse_error(
        format!("Expected expression, got {:?}", self.peek().kind),
        self.peek().position.clone(),
    ))
}

/// Parse array literals or SIMD vector literals starting with [
fn parse_array_or_simd_literal(&mut self) -> Result<Expr> {
    let mut elements = Vec::new();
    
    // Handle empty array/vector
    if self.check(&TokenKind::RightBracket) {
        self.advance(); // consume ]
        return Ok(Expr::Literal(Literal::Vector { 
            elements, 
            vector_type: None 
        }));
    }
    
    // Parse elements
    loop {
        elements.push(self.expression()?);
        
        if !self.match_tokens(&[TokenKind::Comma]) {
            break;
        }
    }
    
    self.consume(TokenKind::RightBracket, "Expected ']' after array elements".to_string())?;
    
    // Check if this is a SIMD vector literal with type annotation
    if self.check_simd_type() {
        let simd_type = self.advance().lexeme.clone();
        return Ok(Expr::SIMD(SIMDExpr::VectorLiteral {
            elements,
            vector_type: Some(simd_type),
            position: self.previous().position.clone(),
        }));
    }
    
    // Regular array literal
    Ok(Expr::Literal(Literal::Vector { 
        elements, 
        vector_type: None 
    }))
}

/// Check if current token is a SIMD type
fn check_simd_type(&self) -> bool {
    if self.is_at_end() { return false; }
    
    matches!(self.peek().kind,
        TokenKind::F32x2 | TokenKind::F32x4 | TokenKind::F32x8 | TokenKind::F32x16 |
        TokenKind::F64x2 | TokenKind::F64x4 | TokenKind::F64x8 |
        TokenKind::I32x2 | TokenKind::I32x4 | TokenKind::I32x8 | TokenKind::I32x16 |
        TokenKind::I64x2 | TokenKind::I64x4 | TokenKind::I64x8 |
        TokenKind::I16x4 | TokenKind::I16x8 | TokenKind::I16x16 | TokenKind::I16x32 |
        TokenKind::I8x8 | TokenKind::I8x16 | TokenKind::I8x32 | TokenKind::I8x64 |
        TokenKind::U32x4 | TokenKind::U32x8 | TokenKind::U16x8 | TokenKind::U16x16 |
        TokenKind::U8x16 | TokenKind::U8x32 |
        TokenKind::Mask8 | TokenKind::Mask16 | TokenKind::Mask32 | TokenKind::Mask64
    )
}

/// Enhanced call parsing that handles both function calls and array indexing
fn call(&mut self) -> Result<Expr> {
    let mut expr = self.primary()?;
    
    loop {
        if self.match_tokens(&[TokenKind::LeftParen]) {
            // Function call
            let mut arguments = Vec::new();
            
            if !self.check(&TokenKind::RightParen) {
                loop {
                    arguments.push(self.expression()?);
                    if !self.match_tokens(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }
            
            self.consume(TokenKind::RightParen, "Expected ')' after arguments".to_string())?;
            expr = Expr::Call(Box::new(expr), arguments);
            
        } else if self.match_tokens(&[TokenKind::LeftBracket]) {
            // Array indexing
            let index = self.expression()?;
            self.consume(TokenKind::RightBracket, "Expected ']' after array index".to_string())?;
            expr = Expr::Index(Box::new(expr), Box::new(index));
            
        } else if self.match_tokens(&[TokenKind::Dot]) {
            // Field access
            let field = self.consume_identifier("Expected field name after '.'".to_string())?;
            expr = Expr::FieldAccess(Box::new(expr), field);
            
        } else {
            break;
        }
    }
    
    Ok(expr)
}

/// Helper function to match tokens and get the matched token
fn match_tokens_and_get(&mut self, types: &[TokenKind]) -> Option<Token> {
    for token_type in types {
        if self.check_token_type(token_type) {
            return Some(self.advance().clone());
        }
    }
    None
}

/// Check if current token matches a specific token type (handles variants)
fn check_token_type(&self, token_type: &TokenKind) -> bool {
    if self.is_at_end() { return false; }
    
    match (token_type, &self.peek().kind) {
        (TokenKind::Integer(_), TokenKind::Integer(_)) => true,
        (TokenKind::Float(_), TokenKind::Float(_)) => true,
        (TokenKind::StringLiteral(_), TokenKind::StringLiteral(_)) => true,
        (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
        (TokenKind::SimdLiteral(_), TokenKind::SimdLiteral(_)) => true,
        (a, b) => a == b,
    }
}

// Additional test fixes for the failing tests
#[cfg(test)]
mod parser_simd_tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_array_indexing_fixed() {
        let source = "array[1 + 2]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);
        
        let expr = parser.expression().expect("Parsing should succeed");
        
        // Verify the structure
        match expr {
            Expr::Index(array, index) => {
                assert!(matches!(**array, Expr::Variable(_)));
                assert!(matches!(**index, Expr::Binary(_, _, _)));
            }
            _ => panic!("Expected Index expression, got {:?}", expr),
        }
    }

    #[test] 
    fn test_simd_vector_literal_parsing() {
        let source = "[1.0, 2.0, 3.0, 4.0]f32x4";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);
        
        let expr = parser.expression().expect("Parsing should succeed");
        
        // Verify SIMD vector literal structure
        match expr {
            Expr::SIMD(SIMDExpr::VectorLiteral { elements, vector_type, .. }) => {
                assert_eq!(elements.len(), 4);
                assert_eq!(vector_type, Some("f32x4".to_string()));
            }
            _ => panic!("Expected SIMD VectorLiteral, got {:?}", expr),
        }
    }

    #[test]
    fn test_regular_array_literal() {
        let source = "[1, 2, 3]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);
        
        let expr = parser.expression().expect("Parsing should succeed");
        
        match expr {
            Expr::Literal(Literal::Vector { elements, vector_type }) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(vector_type, None);
            }
            _ => panic!("Expected Vector literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_empty_array() {
        let source = "[]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all().expect("Lexing should succeed");
        let mut parser = Parser::new(tokens);
        
        let expr = parser.expression().expect("Parsing should succeed");
        
        match expr {
            Expr::Literal(Literal::Vector { elements, vector_type }) => {
                assert_eq!(elements.len(), 0);
                assert_eq!(vector_type, None);
            }
            _ => panic!("Expected empty Vector literal, got {:?}", expr),
        }
    }
}