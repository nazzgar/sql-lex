use crate::{ComparisonOperator, Expr, ParseError, Token, TokenKind, Value, lex};

pub fn parse_where(input: &str) -> Result<Expr, ParseError> {
    let tokens = lex(input)?;
    let mut parser = Parser { tokens, current: 0 };
    let expression = parser.expression()?;
    if !matches!(parser.peek().kind, TokenKind::Eof) {
        return Err(ParseError::new(
            parser.peek().position,
            "unexpected trailing input",
        ));
    }
    Ok(expression)
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.or_expression()
    }

    fn or_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.and_expression()?;
        while self.matches(|kind| matches!(kind, TokenKind::Or)) {
            expression = Expr::or(expression, self.and_expression()?);
        }
        Ok(expression)
    }

    fn and_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.primary()?;
        while self.matches(|kind| matches!(kind, TokenKind::And)) {
            expression = Expr::and(expression, self.primary()?);
        }
        Ok(expression)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(|kind| matches!(kind, TokenKind::LeftParen)) {
            let expression = self.expression()?;
            self.expect(
                |kind| matches!(kind, TokenKind::RightParen),
                "expected ')' after expression",
            )?;
            Ok(expression)
        } else {
            self.comparison()
        }
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let table = self.identifier("expected table identifier")?;
        self.expect(
            |kind| matches!(kind, TokenKind::Dot),
            "expected '.' in column reference",
        )?;
        let column = self.identifier("expected column identifier")?;
        let operator = self.operator()?;
        let value = if operator == ComparisonOperator::In {
            self.list()?
        } else {
            self.value()?
        };
        Ok(Expr::comparison_value(table, column, operator, value))
    }

    fn operator(&mut self) -> Result<ComparisonOperator, ParseError> {
        let operator = match self.peek().kind {
            TokenKind::Equals => ComparisonOperator::Equal,
            TokenKind::NotEqual => ComparisonOperator::NotEqual,
            TokenKind::LessThan => ComparisonOperator::LessThan,
            TokenKind::LessThanOrEqual => ComparisonOperator::LessThanOrEqual,
            TokenKind::GreaterThan => ComparisonOperator::GreaterThan,
            TokenKind::GreaterThanOrEqual => ComparisonOperator::GreaterThanOrEqual,
            TokenKind::In => ComparisonOperator::In,
            _ => {
                return Err(ParseError::new(
                    self.peek().position,
                    "expected comparison operator after column reference",
                ));
            }
        };
        self.advance();
        Ok(operator)
    }

    fn value(&mut self) -> Result<Value, ParseError> {
        match &self.peek().kind {
            TokenKind::String(value) => {
                let value = Value::String(value.clone());
                self.advance();
                Ok(value)
            }
            TokenKind::Integer(value) => {
                let value = Value::Integer(*value);
                self.advance();
                Ok(value)
            }
            TokenKind::Identifier(_) => {
                let table = self.identifier("expected table identifier after '='")?;
                self.expect(
                    |kind| matches!(kind, TokenKind::Dot),
                    "expected '.' in column reference after '='",
                )?;
                let column = self.identifier("expected column identifier after '='")?;
                Ok(Value::Column { table, column })
            }
            _ => Err(ParseError::new(
                self.peek().position,
                "expected string, integer, or column reference after operator",
            )),
        }
    }

    fn list(&mut self) -> Result<Value, ParseError> {
        self.expect(
            |kind| matches!(kind, TokenKind::LeftParen),
            "expected '(' after IN",
        )?;
        let mut values = vec![self.list_value()?];
        while self.matches(|kind| matches!(kind, TokenKind::Comma)) {
            values.push(self.list_value()?);
        }
        self.expect(
            |kind| matches!(kind, TokenKind::RightParen),
            "expected ')' after IN list",
        )?;
        Ok(Value::List(values))
    }

    fn list_value(&mut self) -> Result<Value, ParseError> {
        match self.value()? {
            Value::String(value) => Ok(Value::String(value)),
            _ => Err(ParseError::new(
                self.previous().position,
                "IN lists support string literals only",
            )),
        }
    }

    fn identifier(&mut self, message: &'static str) -> Result<String, ParseError> {
        let identifier = match &self.peek().kind {
            TokenKind::Identifier(identifier) => identifier.clone(),
            _ => return Err(ParseError::new(self.peek().position, message)),
        };
        self.advance();
        Ok(identifier)
    }

    fn expect(
        &mut self,
        predicate: impl FnOnce(&TokenKind) -> bool,
        message: &'static str,
    ) -> Result<(), ParseError> {
        if self.matches(predicate) {
            Ok(())
        } else {
            Err(ParseError::new(self.peek().position, message))
        }
    }

    fn matches(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        if predicate(&self.peek().kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> &Token {
        if !matches!(self.peek().kind, TokenKind::Eof) {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }
}
