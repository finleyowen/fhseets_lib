use std::{fs, rc::Rc};

use super::{
    core::*,
    json::ToJson,
    lex::*,
    parse::*,
    validate::{Validate, validate_prgm},
};
use rlrl::prelude::*;

fn lex(s: &str) -> anyhow::Result<TokenQueue<Token>> {
    let lexer = setup_lexer();
    let tokens = lexer.lex(s)?;
    let tq = TokenQueue::from(tokens);
    Ok(tq)
}

fn maps_to_int_range(s: &str) -> anyhow::Result<bool> {
    let out = parse_int_range(&lex(&s)?)?.0.to_string();
    Ok(out == s)
}

fn maps_to_dbl_range(s: &str) -> anyhow::Result<bool> {
    let out = parse_dbl_range(&lex(&s)?)?.0.to_string();
    Ok(out == s)
}

fn maps_to_data_type(s: &str) -> anyhow::Result<bool> {
    let out = parse_dtype(&lex(&s)?)?.0.to_string();
    Ok(out == s)
}

fn maps_to_column_schema(s: &str) -> anyhow::Result<bool> {
    let out = parse_column_schema(&lex(&s)?)?.0.to_string();
    Ok(out == s)
}

fn parse_prgm_from_str(s: &str) -> anyhow::Result<DbSchema> {
    let mut tq = lex(s)?;
    let prgm = tq.parse(parse_prgm)?;
    Ok(prgm)
}

#[test]
fn int_range_test() -> anyhow::Result<()> {
    assert!(maps_to_int_range("<5,10>")?);
    assert!(maps_to_int_range("<10,10>")?);
    assert!(maps_to_int_range("<,10>")?);
    assert!(maps_to_int_range("<5,>")?);

    Ok(())
}

#[test]
fn dbl_range_test() -> anyhow::Result<()> {
    assert!(maps_to_dbl_range("<5,>")?);
    assert!(maps_to_dbl_range("<,10>")?);
    assert!(maps_to_dbl_range("<1.1,9.9>")?);
    assert!(maps_to_dbl_range("<1.1,9.900009>")?);

    Ok(())
}

#[test]
fn data_type_test() -> anyhow::Result<()> {
    assert!(maps_to_data_type("int<5,10>")?);
    assert!(maps_to_data_type("int<5,10>?")?);
    assert!(maps_to_data_type("str<5,10>")?);
    assert!(maps_to_data_type("str<5,10>?")?);
    assert!(maps_to_data_type("dbl<5.4,10.3>")?);
    assert!(maps_to_data_type("dbl<5,10>?")?);
    assert!(maps_to_data_type("int1to5?")?);
    Ok(())
}

#[test]
fn column_schema_test() -> anyhow::Result<()> {
    assert!(maps_to_column_schema("abc: int<5,5>")?);
    assert!(maps_to_column_schema("\"My column\": int<5,5>")?);
    assert!(maps_to_column_schema("\"My optional column\": int<5,5>?")?);

    Ok(())
}

#[test]
fn stmt_test() -> anyhow::Result<()> {
    let mut tq = lex("type i1to5 int<1,5>")?;
    let stmt = tq.parse(parse_stmt)?;
    assert!(
        stmt == Stmt::TypeDef(
            "i1to5".into(),
            Rc::new(ParentTypeExpr::Int(Range {
                min: Some(1),
                max: Some(5)
            }))
        )
    );

    let tokens = setup_lexer().lex("table Table1(a: int?, b: dbl?, c:str?)")?;
    assert!(parse_stmt(&TokenQueue::from(tokens)).is_ok());

    Ok(())
}

#[test]
fn ident_test() -> anyhow::Result<()> {
    let ident = "helloWorld";
    let ident_tok = Token::Ident(ident.into());
    assert!(
        ident_tok.is_ident_or_str_literal_tok()
            && ident_tok.get_ident_or_str_literal() == Some(ident),
    );

    let str_lit = "'helloWorld'";
    let str_lit_tok = Token::Literal(Literal::Str(str_lit.into()));
    assert!(
        str_lit_tok.is_ident_or_str_literal_tok()
            && str_lit_tok.get_ident_or_str_literal() == Some("helloWorld")
    );

    let str_lit = "''";
    let str_lit_tok = Token::Literal(Literal::Str(str_lit.into()));
    assert!(str_lit_tok.get_ident_or_str_literal() == Some(""));

    Ok(())
}

#[test]
fn table_schema_test() -> anyhow::Result<()> {
    let mut tq = lex("Table(
            col1: int<1, 5>, 
            col2: dbl<1, 5>, 
            col3: str<5, 10>,
            col4: int<1, 6>?
        )")?;

    let table = tq.parse(parse_table_schema)?;

    assert!(
        table
            == TableSchemaExpr {
                table_name: "Table".into(),
                columns: vec![
                    ColumnSchemaExpr {
                        column_name: "col1".into(),
                        dtype: DTypeExpr {
                            parent: ParentTypeExpr::Int(Range {
                                min: Some(1),
                                max: Some(5)
                            }),
                            nullable: false
                        },
                        default_value: None
                    },
                    ColumnSchemaExpr {
                        column_name: "col2".into(),
                        dtype: DTypeExpr {
                            parent: ParentTypeExpr::Dbl(Range {
                                min: Some(1.0),
                                max: Some(5.0)
                            }),
                            nullable: false
                        },
                        default_value: None
                    },
                    ColumnSchemaExpr {
                        column_name: "col3".into(),
                        dtype: DTypeExpr {
                            parent: ParentTypeExpr::Str(Range {
                                min: Some(5),
                                max: Some(10)
                            }),
                            nullable: false
                        },
                        default_value: None
                    },
                    ColumnSchemaExpr {
                        column_name: "col4".into(),
                        dtype: DTypeExpr {
                            parent: ParentTypeExpr::Int(Range {
                                min: Some(1),
                                max: Some(6)
                            }),
                            nullable: true
                        },
                        default_value: None
                    }
                ]
            }
    );

    Ok(())
}

#[test]
fn parse_prgm_test() -> anyhow::Result<()> {
    let prgm = parse_prgm_from_str(include_str!(
        "../../test_artifacts/valid_schemas/valid_schema_1.txt"
    ))?;
    assert!(prgm.stmts.len() == 6);

    let prgm = parse_prgm_from_str(include_str!(
        "../../test_artifacts/valid_schemas/valid_schema_2.txt"
    ))?;
    assert!(prgm.stmts.len() == 4);

    Ok(())
}

fn load_prgm(path: &str) -> anyhow::Result<DbSchema> {
    parse_prgm_from_str(&fs::read_to_string(path)?)
}

#[test]
fn validate_prgm_test() -> anyhow::Result<()> {
    let prgm =
        load_prgm("test_artifacts/invalid_schemas/invalid_schema_1.txt")?;
    assert!(validate_prgm(&prgm).is_err());

    let prgm =
        load_prgm("test_artifacts/invalid_schemas/invalid_schema_2.txt")?;
    assert!(validate_prgm(&prgm).is_err());

    let prgm =
        load_prgm("test_artifacts/invalid_schemas/invalid_schema_3.txt")?;
    assert!(validate_prgm(&prgm).is_err());

    let prgm =
        load_prgm("test_artifacts/invalid_schemas/invalid_schema_4.txt")?;
    assert!(validate_prgm(&prgm).is_err());

    Ok(())
}

#[test]
fn json_test() -> anyhow::Result<()> {
    let ss_schema = parse_prgm_from_str("type myType int<1,5>;")?;
    let mut sym_table = SymbolTable::new();
    ss_schema.validate(&mut sym_table)?;
    let val = ss_schema.to_json(&sym_table)?;
    dbg!(val);

    Ok(())
}
