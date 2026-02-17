use super::core::*;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

const STR_LEN_MSG: &str = "String length can't be negative!";

pub trait Validate {
    fn validate(&self, sym_table: &mut SymbolTable) -> anyhow::Result<()>;
}

impl<T> Validate for Range<T>
where
    T: Display + PartialOrd + Clone,
{
    fn validate(&self, _: &mut SymbolTable) -> anyhow::Result<()> {
        if let Some(min) = self.min.clone()
            && let Some(max) = self.max.clone()
            && min > max
        {
            return Err(anyhow::anyhow!(
                "Can't have min > max ({min} > {max})"
            ));
        }
        Ok(())
    }
}

impl Validate for ParentTypeExpr {
    fn validate(&self, sym_table: &mut SymbolTable) -> anyhow::Result<()> {
        match &self {
            ParentTypeExpr::Int(range) => range.validate(sym_table),
            ParentTypeExpr::Dbl(range) => range.validate(sym_table),
            ParentTypeExpr::Str(range) => {
                range.validate(sym_table)?;
                if let Some(min) = range.min
                    && min < 0
                {
                    return Err(anyhow::anyhow!(STR_LEN_MSG));
                }

                if let Some(max) = range.max
                    && max < 0
                {
                    return Err(anyhow::anyhow!(STR_LEN_MSG));
                }

                Ok(())
            }
            ParentTypeExpr::Ident(ident) => {
                if !sym_table.contains_key(ident) {
                    return Err(anyhow::anyhow!("Unrecognised dtype {ident}"));
                }
                Ok(())
            }
        }
    }
}

impl Validate for ColumnSchemaExpr {
    fn validate(&self, sym_table: &mut SymbolTable) -> anyhow::Result<()> {
        self.dtype.parent.validate(sym_table)?;
        Ok(())
    }
}

impl Validate for TableSchemaExpr {
    fn validate(&self, sym_table: &mut SymbolTable) -> anyhow::Result<()> {
        let mut column_names: HashSet<&str> = HashSet::new();
        column_names.reserve(self.columns.len());

        for column in &self.columns {
            let column_name = &column.column_name;
            if !column_names.insert(column_name) {
                return Err(anyhow::anyhow!(
                    "Can't have two columns named {column_name}!"
                ));
            }

            column.validate(sym_table)?;
        }

        Ok(())
    }
}

impl Validate for Stmt {
    fn validate(&self, sym_table: &mut SymbolTable) -> anyhow::Result<()> {
        match self {
            Self::TypeDef(name, parent_type) => {
                if let Some(_) = sym_table
                    .insert(name.clone(), Symbol::TypeDef(parent_type.clone()))
                {
                    return Err(anyhow::anyhow!(
                        "Can't have two defined types named {name}"
                    ));
                }
                parent_type.validate(sym_table)?;
                return Ok(());
            }
            Self::TableSchema(table) => {
                if let Some(_) = sym_table.insert(
                    table.table_name.clone(),
                    Symbol::Table(table.clone()),
                ) {
                    return Err(anyhow::anyhow!(
                        "Can't have two tables named {}",
                        table.table_name
                    ));
                }
                table.validate(sym_table)?;
                return Ok(());
            }
        }
    }
}

impl Validate for DbSchema {
    fn validate(&self, sym_table: &mut SymbolTable) -> anyhow::Result<()> {
        for stmt in &self.stmts {
            stmt.validate(sym_table)?;
        }
        Ok(())
    }
}

pub fn validate_prgm(prgm: &DbSchema) -> anyhow::Result<SymbolTable> {
    let mut sym_table = HashMap::new();
    prgm.validate(&mut sym_table)?;
    Ok(sym_table)
}
