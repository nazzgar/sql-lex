use crate::types::{ParseError, Token, TokenKind};

pub fn lex(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut characters = input.char_indices().peekable();

    while let Some((position, character)) = characters.next() {
        match character {
            character if character.is_ascii_whitespace() => {}
            '.' => tokens.push(Token::new(TokenKind::Dot, position)),
            '=' => tokens.push(Token::new(TokenKind::Equals, position)),
            '<' => tokens.push(comparison_token(
                &mut characters,
                position,
                TokenKind::LessThan,
                TokenKind::LessThanOrEqual,
            )),
            '>' => tokens.push(comparison_token(
                &mut characters,
                position,
                TokenKind::GreaterThan,
                TokenKind::GreaterThanOrEqual,
            )),
            ',' => tokens.push(Token::new(TokenKind::Comma, position)),
            '(' => tokens.push(Token::new(TokenKind::LeftParen, position)),
            ')' => tokens.push(Token::new(TokenKind::RightParen, position)),
            '\'' => tokens.push(string_token(&mut characters, position)?),
            character if is_identifier_start(character) => {
                tokens.push(identifier_token(&mut characters, position, character));
            }
            character if character.is_ascii_digit() => {
                tokens.push(number_token(&mut characters, position, character)?);
            }
            _ => {
                return Err(ParseError::new(
                    position,
                    format!("unexpected character {character:?}"),
                ));
            }
        }
    }

    tokens.push(Token::new(TokenKind::Eof, input.len()));
    Ok(tokens)
}

fn comparison_token(
    characters: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    position: usize,
    single: TokenKind,
    inclusive: TokenKind,
) -> Token {
    let kind = if matches!(characters.peek(), Some((_, '='))) {
        characters.next();
        inclusive
    } else if matches!(single, TokenKind::LessThan) && matches!(characters.peek(), Some((_, '>'))) {
        characters.next();
        TokenKind::NotEqual
    } else {
        single
    };
    Token::new(kind, position)
}

fn number_token(
    characters: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    position: usize,
    first: char,
) -> Result<Token, ParseError> {
    let mut number = String::from(first);
    while let Some((_, next)) = characters.peek() {
        if !next.is_ascii_digit() {
            break;
        }
        number.push(*next);
        characters.next();
    }
    let value = number
        .parse()
        .map_err(|_| ParseError::new(position, "integer literal is too large"))?;
    Ok(Token::new(TokenKind::Integer(value), position))
}

fn string_token(
    characters: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    position: usize,
) -> Result<Token, ParseError> {
    let mut value = String::new();
    while let Some((_, character)) = characters.next() {
        if character != '\'' {
            value.push(character);
        } else if matches!(characters.peek(), Some((_, '\''))) {
            characters.next();
            value.push('\'');
        } else {
            return Ok(Token::new(TokenKind::String(value), position));
        }
    }
    Err(ParseError::new(position, "unterminated string literal"))
}

fn identifier_token(
    characters: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    position: usize,
    first: char,
) -> Token {
    let mut identifier = String::from(first);
    while let Some((_, next)) = characters.peek() {
        if !is_identifier_continue(*next) {
            break;
        }
        identifier.push(*next);
        characters.next();
    }
    let kind = if identifier.eq_ignore_ascii_case("AND") {
        TokenKind::And
    } else if identifier.eq_ignore_ascii_case("OR") {
        TokenKind::Or
    } else if identifier.eq_ignore_ascii_case("IN") {
        TokenKind::In
    } else {
        TokenKind::Identifier(identifier)
    };
    Token::new(kind, position)
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit()
}
