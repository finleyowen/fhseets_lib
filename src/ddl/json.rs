use super::core::*;
use serde_json::json;

pub trait ToJson {
    fn to_json(
        &self,
        sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value>;
}

impl ToJson for ParentTypeExpr {
    fn to_json(
        &self,
        sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value> {
        match self {
            Self::Int(r) => Ok(json!({
                "type": "int",
                "min": r.min,
                "max": r.max
            })),
            Self::Dbl(r) => Ok(json!({
                "type": "dbl",
                "min": r.min,
                "max": r.max
            })),
            Self::Str(r) => Ok(json!({
                "type": "str",
                "min": r.min,
                "max": r.max
            })),
            Self::Ident(ident) => sym_table
                .get(ident)
                .ok_or(anyhow::anyhow!("Unkown ident {ident}"))?
                .to_json(&sym_table),
        }
    }
}

impl ToJson for DTypeExpr {
    fn to_json(
        &self,
        sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value> {
        Ok(
            json!({ "nullable": self.nullable, "parent": self.parent.to_json(sym_table)? }),
        )
    }
}

impl ToJson for ColumnSchemaExpr {
    fn to_json(
        &self,
        sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value> {
        Ok(json!({
            "column_name": self.column_name,
            "dtype": self.dtype.to_json(sym_table)?
        }))
    }
}

impl ToJson for TableSchemaExpr {
    fn to_json(
        &self,
        _sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value> {
        Ok(json!({"table_name": self.table_name}))
    }
}

impl ToJson for Symbol {
    fn to_json(
        &self,
        sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value> {
        match &self {
            Self::TypeDef(parent_type) => parent_type.to_json(sym_table),
            Self::Table(table_schema) => table_schema.to_json(sym_table),
        }
    }
}

impl ToJson for Stmt {
    fn to_json(
        &self,
        sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value> {
        match self {
            Stmt::TableSchema(table_schema) => Ok(json!({
                "tableSchema": table_schema.to_json(sym_table)?
            })),
            Stmt::TypeDef(name, dytpe) => Ok(json!({
                "typeDef": {name: dytpe.to_json(sym_table)?}
            })),
        }
    }
}

impl ToJson for DbSchema {
    fn to_json(
        &self,
        sym_table: &SymbolTable,
    ) -> anyhow::Result<serde_json::Value> {
        let mut stmts: Vec<serde_json::Value> = vec![];
        for stmt in &self.stmts {
            stmts.push(stmt.to_json(sym_table)?);
        }
        let stmts = serde_json::Value::Array(stmts);
        Ok(json!({"stmts": stmts}))
    }
}
