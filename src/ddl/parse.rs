use super::core::*;
use super::lex::*;
use rlrl::parse::*;
use std::rc::Rc;

pub fn parse_int_range(tq: &TokenQueue<Token>) -> ParseResult<IntRange> {
    // create a mutable copy
    let mut tq = tq.clone();

    tq.consume_eq(Token::OAngle)?;

    // consume min
    let min = match tq.clone().peek_matching(|token| token.is_literal()) {
        Ok(token) => {
            tq.increment();
            let literal = token.get_literal().unwrap();
            if literal.is_int_literal() {
                Some(literal.get_int().unwrap())
            } else {
                return Err(anyhow::anyhow!("Couldn't parse int literal!"));
            }
        }
        Err(_) => None,
    };

    tq.consume_eq(Token::Comma)?;

    // consume max
    let max = match tq.clone().peek_matching(|token| token.is_literal()) {
        Ok(token) => {
            tq.increment();
            let literal = token.get_literal().unwrap();
            if literal.is_int_literal() {
                Some(literal.get_int().unwrap())
            } else {
                return Err(anyhow::anyhow!("Couldn't parse int literal!"));
            }
        }
        Err(_) => None,
    };

    tq.consume_eq(Token::CAngle)?;

    Ok((IntRange { min, max }, tq.get_idx()))
}

pub fn parse_dbl_range(tq: &TokenQueue<Token>) -> ParseResult<DblRange> {
    let mut tq = tq.clone();

    // consume '<'
    tq.consume_eq(Token::OAngle)?;

    // consume min
    let min = match tq.clone().peek_matching(|token| token.is_literal()) {
        Ok(token) => {
            tq.increment();
            let literal = token.get_literal().unwrap();
            if literal.is_dbl_literal() {
                Some(literal.get_dbl().unwrap())
            } else {
                return Err(anyhow::anyhow!("Couldn't parse dbl literal!"));
            }
        }
        Err(_) => None,
    };

    // consume ','
    tq.consume_eq(Token::Comma)?;

    // consume max
    let max = match tq.clone().peek_matching(|token| token.is_literal()) {
        Ok(token) => {
            tq.increment();
            let literal = token.get_literal().unwrap();
            if literal.is_dbl_literal() {
                Some(literal.get_dbl().unwrap())
            } else {
                return Err(anyhow::anyhow!("Couldn't parse dbl literal!"));
            }
        }
        Err(_) => None,
    };

    // consume '>'
    tq.consume_eq(Token::CAngle)?;

    // done
    Ok((DblRange { min, max }, tq.get_idx()))
}

pub fn parse_parent_type(
    tq: &TokenQueue<Token>,
) -> ParseResult<ParentTypeExpr> {
    let mut tq = tq.clone();

    let parent_name = tq
        .consume_matching(|tok| tok.is_ident_or_str_literal_tok())?
        .get_ident_or_str_literal()
        .ok_or(anyhow::anyhow!("Couldn't get type name"))?;

    match parent_name.to_lowercase().as_str() {
        "int" | "integer" => {
            if let Ok((range, end)) = parse_int_range(&tq) {
                return Ok((ParentTypeExpr::Int(range), end));
            }
            return Ok((
                ParentTypeExpr::Int(Range {
                    min: None,
                    max: None,
                }),
                tq.get_idx(),
            ));
        }
        "str" | "string" | "text" => {
            if let Ok((range, end)) = parse_int_range(&tq) {
                return Ok((ParentTypeExpr::Str(range), end));
            }
            return Ok((
                ParentTypeExpr::Str(Range {
                    min: None,
                    max: None,
                }),
                tq.get_idx(),
            ));
        }
        "dbl" | "double" | "float" => {
            if let Ok((range, end)) = parse_dbl_range(&tq) {
                return Ok((ParentTypeExpr::Dbl(range), end));
            }
            return Ok((
                ParentTypeExpr::Dbl(Range {
                    min: None,
                    max: None,
                }),
                tq.get_idx(),
            ));
        }
        _ => Ok((ParentTypeExpr::Ident(parent_name.to_string()), tq.get_idx())),
    }
}

pub fn parse_dtype(tq: &TokenQueue<Token>) -> ParseResult<DTypeExpr> {
    let mut tq = tq.clone();

    let parent = tq.parse(parse_parent_type)?;

    let nullable = tq.consume_eq(Token::QMark).is_ok();

    Ok((DTypeExpr { parent, nullable }, tq.get_idx()))
}

pub fn parse_column_schema(
    tq: &TokenQueue<Token>,
) -> ParseResult<ColumnSchemaExpr> {
    let mut tq: TokenQueue<Token> = tq.clone();

    let column_name = tq
        .consume_matching(|tok| tok.is_ident_or_str_literal_tok())?
        .get_ident_or_str_literal()
        .ok_or(anyhow::anyhow!("Couldn't get column name!"))?
        .to_string();

    tq.consume_eq(Token::Colon)?;

    let dtype = tq.parse(parse_dtype)?;

    let mut default_value = None;
    if tq.consume_eq(Token::Equals).is_ok() {
        default_value = Some(
            tq.consume_matching(|tok| tok.is_literal())?
                .get_literal()
                .ok_or(anyhow::anyhow!(
                    "Couldn't get default value literal!"
                ))?,
        );
    }

    Ok((
        ColumnSchemaExpr {
            column_name,
            dtype,
            default_value: default_value.cloned().into(),
        },
        tq.get_idx(),
    ))
}

pub fn parse_table_schema(
    tq: &TokenQueue<Token>,
) -> ParseResult<TableSchemaExpr> {
    let mut tq_mut = tq.clone();

    let table_name = tq
        .peek_matching(|tok| tok.is_ident_or_str_literal_tok())?
        .get_ident_or_str_literal()
        .ok_or(anyhow::anyhow!("Couldn't get table name!"))?;
    tq_mut.increment();

    tq_mut.consume_eq(Token::OParen)?;

    let mut columns = vec![];

    while let Ok(column) = tq_mut.parse(parse_column_schema) {
        columns.push(column);
        if tq_mut.consume_eq(Token::Comma).is_err() {
            break;
        }
    }

    tq_mut.consume_eq(Token::CParen)?;
    Ok((
        TableSchemaExpr {
            table_name: table_name.to_string(),
            columns,
        },
        tq_mut.get_idx(),
    ))
}

pub fn parse_stmt(tq: &TokenQueue<Token>) -> ParseResult<Stmt> {
    let mut tq = tq.clone();

    match tq.consume() {
        Ok(Token::TypeKwd) => {
            let type_name = tq
                .consume_matching(|tok| tok.is_ident_or_str_literal_tok())?
                .get_ident_or_str_literal()
                .ok_or(anyhow::anyhow!("Couldn't get type name!"))?
                .to_string();

            let (parent_type, end) = parse_parent_type(&tq)?;
            // tq.consume_eq(Token::)
            Ok((Stmt::TypeDef(type_name.into(), Rc::new(parent_type)), end))
        }
        Ok(Token::TableKwd) => {
            let (table_schema, end) = parse_table_schema(&tq)?;
            Ok((Stmt::TableSchema(Rc::new(table_schema)), end))
        }
        Ok(tok) => {
            dbg!(tok);
            Err(anyhow::anyhow!("Couldn't parse statement!"))
        }
        Err(_) => Err(anyhow::anyhow!("Couldn't parse statement!")),
    }
}

pub fn parse_prgm(tq: &TokenQueue<Token>) -> ParseResult<DbSchema> {
    let mut tq = tq.clone();
    let mut stmts = vec![];

    while let Ok((stmt, end)) = parse_stmt(&tq) {
        tq.go_to(end);
        stmts.push(stmt);
        if tq.consume_eq(Token::Semicolon).is_err() {
            return Err(anyhow::anyhow!("Missing semicolon!"));
        }
    }

    if !tq.is_consumed() {
        dbg!(&tq);
        return Err(anyhow::anyhow!("Program ends without a valid statement"));
    }

    Ok((DbSchema { stmts }, tq.get_idx()))
}
