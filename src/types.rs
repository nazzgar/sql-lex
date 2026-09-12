use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    String(String),
    Integer(i64),
    And,
    Or,
    In,
    Dot,
    Equals,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Comma,
    LeftParen,
    RightParen,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: usize,
}

impl Token {
    pub fn new(kind: TokenKind, position: usize) -> Self {
        Self { kind, position }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub message: String,
}

impl ParseError {
    pub(crate) fn new(position: usize, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.position)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Comparison {
        table: String,
        column: String,
        operator: ComparisonOperator,
        value: Value,
    },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Integer(i64),
    Column { table: String, column: String },
    List(Vec<Value>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    In,
}

impl Expr {
    pub fn comparison(
        table: impl Into<String>,
        column: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::Comparison {
            table: table.into(),
            column: column.into(),
            operator: ComparisonOperator::Equal,
            value: Value::String(value.into()),
        }
    }

    pub fn comparison_column(
        table: impl Into<String>,
        column: impl Into<String>,
        value_table: impl Into<String>,
        value_column: impl Into<String>,
    ) -> Self {
        Self::Comparison {
            table: table.into(),
            column: column.into(),
            operator: ComparisonOperator::Equal,
            value: Value::Column {
                table: value_table.into(),
                column: value_column.into(),
            },
        }
    }

    pub fn comparison_value(
        table: impl Into<String>,
        column: impl Into<String>,
        operator: ComparisonOperator,
        value: Value,
    ) -> Self {
        Self::Comparison {
            table: table.into(),
            column: column.into(),
            operator,
            value,
        }
    }

    pub fn and(left: Expr, right: Expr) -> Self {
        Self::And(Box::new(left), Box::new(right))
    }

    pub fn or(left: Expr, right: Expr) -> Self {
        Self::Or(Box::new(left), Box::new(right))
    }
}
