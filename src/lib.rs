pub mod lexer;
pub mod parser;
pub mod types;

pub use lexer::lex;
pub use parser::parse_where;
pub use types::{ComparisonOperator, Expr, ParseError, Token, TokenKind, Value};

#[cfg(test)]
mod tests {
    use crate::{ComparisonOperator, Expr, Token, TokenKind, Value, lex, parse_where};

    #[test]
    fn lexes_a_comparison() {
        assert_eq!(
            lex("users.name = 'Ada'").unwrap(),
            vec![
                Token::new(TokenKind::Identifier("users".into()), 0),
                Token::new(TokenKind::Dot, 5),
                Token::new(TokenKind::Identifier("name".into()), 6),
                Token::new(TokenKind::Equals, 11),
                Token::new(TokenKind::String("Ada".into()), 13),
                Token::new(TokenKind::Eof, 18),
            ]
        );
    }

    #[test]
    fn lexes_keywords_regardless_of_case() {
        assert_eq!(
            lex("a.b='x' and c.d='y' Or e.f='z'").unwrap(),
            vec![
                Token::new(TokenKind::Identifier("a".into()), 0),
                Token::new(TokenKind::Dot, 1),
                Token::new(TokenKind::Identifier("b".into()), 2),
                Token::new(TokenKind::Equals, 3),
                Token::new(TokenKind::String("x".into()), 4),
                Token::new(TokenKind::And, 8),
                Token::new(TokenKind::Identifier("c".into()), 12),
                Token::new(TokenKind::Dot, 13),
                Token::new(TokenKind::Identifier("d".into()), 14),
                Token::new(TokenKind::Equals, 15),
                Token::new(TokenKind::String("y".into()), 16),
                Token::new(TokenKind::Or, 20),
                Token::new(TokenKind::Identifier("e".into()), 23),
                Token::new(TokenKind::Dot, 24),
                Token::new(TokenKind::Identifier("f".into()), 25),
                Token::new(TokenKind::Equals, 26),
                Token::new(TokenKind::String("z".into()), 27),
                Token::new(TokenKind::Eof, 30),
            ]
        );
    }

    #[test]
    fn parses_a_comparison() {
        assert_eq!(
            parse_where("users.name = 'Ada'").unwrap(),
            Expr::comparison("users", "name", "Ada")
        );
    }

    #[test]
    fn parses_a_column_to_column_comparison() {
        assert_eq!(
            parse_where("users.role = users.second_role").unwrap(),
            Expr::comparison_column("users", "role", "users", "second_role")
        );
    }

    #[test]
    fn parses_comparison_operators_and_integer_values() {
        assert_eq!(
            parse_where(
                "users.role = 'admin' AND users.active <> 'yes' and users.some_value > 123 and users.some_value2 < 123 and users.some_value2 <= 123 and users.some_value2 >= 123"
            )
            .unwrap(),
            Expr::and(
                Expr::and(
                    Expr::and(
                        Expr::and(
                            Expr::and(
                                Expr::comparison("users", "role", "admin"),
                                Expr::comparison_value(
                                    "users", "active", ComparisonOperator::NotEqual, Value::String("yes".into())
                                ),
                            ),
                            Expr::comparison_value("users", "some_value", ComparisonOperator::GreaterThan, Value::Integer(123)),
                        ),
                        Expr::comparison_value("users", "some_value2", ComparisonOperator::LessThan, Value::Integer(123)),
                    ),
                    Expr::comparison_value("users", "some_value2", ComparisonOperator::LessThanOrEqual, Value::Integer(123)),
                ),
                Expr::comparison_value("users", "some_value2", ComparisonOperator::GreaterThanOrEqual, Value::Integer(123)),
            )
        );
    }

    #[test]
    fn parses_an_in_list() {
        assert_eq!(
            parse_where("users.role IN ('aba', 'ddsadas')").unwrap(),
            Expr::comparison_value(
                "users",
                "role",
                ComparisonOperator::In,
                Value::List(vec![
                    Value::String("aba".into()),
                    Value::String("ddsadas".into())
                ]),
            )
        );
    }

    #[test]
    fn and_binds_more_tightly_than_or() {
        assert_eq!(
            parse_where("a.x = '1' OR b.y = '2' AND c.z = '3'").unwrap(),
            Expr::or(
                Expr::comparison("a", "x", "1"),
                Expr::and(
                    Expr::comparison("b", "y", "2"),
                    Expr::comparison("c", "z", "3"),
                ),
            )
        );
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_eq!(
            parse_where("(a.x = '1' OR b.y = '2') AND c.z = '3'").unwrap(),
            Expr::and(
                Expr::or(
                    Expr::comparison("a", "x", "1"),
                    Expr::comparison("b", "y", "2"),
                ),
                Expr::comparison("c", "z", "3"),
            )
        );
    }

    #[test]
    fn accepts_escaped_quotes_in_strings() {
        assert_eq!(
            parse_where("people.name = 'O''Brien'").unwrap(),
            Expr::comparison("people", "name", "O'Brien")
        );
    }

    #[test]
    fn rejects_unterminated_strings() {
        let error = lex("users.name = 'Ada").unwrap_err();
        assert_eq!(error.position, 13);
    }

    #[test]
    fn rejects_incomplete_comparisons() {
        let error = parse_where("users.name =").unwrap_err();
        assert_eq!(error.position, 12);
    }

    #[test]
    fn rejects_trailing_input() {
        let error = parse_where("users.name = 'Ada' nonsense").unwrap_err();
        assert_eq!(error.position, 19);
    }
}
