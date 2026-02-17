use std::cmp::min;

use super::core::Literal;
use rlrl::lex::*;

/// Enum representing the tokens available to the lexer.
#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    // chars
    OParen,
    CParen,
    OAngle,
    CAngle,
    Colon,
    Semicolon,
    Comma,
    Equals,
    QMark,

    // kwds
    TypeKwd,
    TableKwd,
    SchemaKwd,

    // ident
    Ident(String),

    // literals
    Literal(Literal),
}

impl Token {
    // helper function for handling identifiers
    pub fn is_ident_or_str_literal_tok(&self) -> bool {
        match self {
            Self::Ident(_) => true,
            Self::Literal(literal) => literal.is_str_literal(),
            _ => false,
        }
    }

    // helper function for handling identifiers
    pub fn get_ident_or_str_literal(&self) -> Option<&str> {
        match &self {
            Self::Ident(ident) => Some(ident),
            Self::Literal(literal) => {
                literal.get_str().map(|s| &s[min(s.len(), 1)..s.len() - 1])
            }
            _ => None,
        }
    }

    pub fn is_literal(&self) -> bool {
        match &self {
            Self::Literal(_) => true,
            _ => false,
        }
    }

    pub fn get_literal(&self) -> Option<&Literal> {
        match &self {
            Self::Literal(literal) => Some(literal),
            _ => None,
        }
    }
}

/// Function to setup the lexer for testing
pub fn setup_lexer() -> Lexer<Token> {
    let mut lexer: Lexer<Token> = Lexer::new();

    // comments
    lexer.add_rule(r"//.*\n", |_| LexResult::Ignore);
    lexer.add_rule(r"/\*[^(\*/)]+\*/", |_| LexResult::Ignore);

    // whitespace
    lexer.add_rule(r"[\s\n\t]+", |_| LexResult::Ignore);

    // chars
    lexer.add_rule(r"\(", |_| LexResult::Token(Token::OParen));
    lexer.add_rule(r"\)", |_| LexResult::Token(Token::CParen));
    lexer.add_rule(r"<", |_| LexResult::Token(Token::OAngle));
    lexer.add_rule(r">", |_| LexResult::Token(Token::CAngle));
    lexer.add_rule(r":", |_| LexResult::Token(Token::Colon));
    lexer.add_rule(r";", |_| LexResult::Token(Token::Semicolon));
    lexer.add_rule(r"\,", |_| LexResult::Token(Token::Comma));
    lexer.add_rule(r"=", |_| LexResult::Token(Token::Equals));
    lexer.add_rule(r"\?", |_| LexResult::Token(Token::QMark));

    // kwds
    lexer.add_rule(r"type", |_| LexResult::Token(Token::TypeKwd));
    lexer.add_rule(r"table", |_| LexResult::Token(Token::TableKwd));

    // idents
    lexer.add_rule(r"[a-zA-Z][a-zA-Z0-9_]*", |re_match| {
        LexResult::Token(Token::Ident(re_match.as_str().into()))
    });

    // literals
    lexer.add_rule(r"\-?[0-9]+", |re_match| {
        match re_match.as_str().parse::<i32>() {
            Ok(v) => LexResult::Token(Token::Literal(Literal::Int(v))),
            Err(e) => LexResult::Error(e.into()),
        }
    });
    lexer.add_rule(r"\-?[0-9]+(\.[0-9]+)?", |re_match| {
        match re_match.as_str().parse::<f64>() {
            Ok(v) => LexResult::Token(Token::Literal(Literal::Dbl(v))),
            Err(e) => LexResult::Error(e.into()),
        }
    });
    lexer.add_rule("\"[^\"]*\"", |re_match| {
        LexResult::Token(Token::Literal(Literal::Str(re_match.as_str().into())))
    });

    lexer.add_rule(".", |re_match| {
        dbg!(re_match.as_str(), re_match.start());
        LexResult::Error(anyhow::anyhow!(
            "Unmatched input at position {}",
            re_match.start()
        ))
    });

    lexer
}
