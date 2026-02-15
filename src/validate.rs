use crate::parse::{ColumnSchema, ParentType, Prgm, Range, Stmt, TableSchema};
use std::{collections::HashSet, fmt::Display};

const STR_LEN_MSG: &str = "String length can't be negative!";

pub struct PrgmValidState {
    pub dtypes: HashSet<String>,
    pub tables: HashSet<String>,
}

pub trait Validate {
    fn validate(&self, valid_state: &mut PrgmValidState) -> anyhow::Result<()>;
}

impl<T> Validate for Range<T>
where
    T: Display + PartialOrd + Clone,
{
    fn validate(&self, _: &mut PrgmValidState) -> anyhow::Result<()> {
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

impl Validate for ParentType {
    fn validate(&self, valid_state: &mut PrgmValidState) -> anyhow::Result<()> {
        match &self {
            ParentType::Int(range) => range.validate(valid_state),
            ParentType::Dbl(range) => range.validate(valid_state),
            ParentType::Str(range) => {
                range.validate(valid_state)?;
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
            ParentType::Ident(ident) => {
                if !valid_state.dtypes.contains(ident) {
                    return Err(anyhow::anyhow!("Unrecognised dtype {ident}"));
                }
                Ok(())
            }
        }
    }
}

impl Validate for ColumnSchema {
    fn validate(&self, valid_state: &mut PrgmValidState) -> anyhow::Result<()> {
        self.dtype.parent.validate(valid_state)?;
        Ok(())
    }
}

impl Validate for TableSchema {
    fn validate(&self, valid_state: &mut PrgmValidState) -> anyhow::Result<()> {
        let mut column_names: HashSet<&str> = HashSet::new();
        column_names.reserve(self.columns.len());

        for column in &self.columns {
            let column_name = &column.column_name;
            if !column_names.insert(column_name) {
                return Err(anyhow::anyhow!(
                    "Can't have two columns named {column_name}!"
                ));
            }

            column.validate(valid_state)?;
        }

        Ok(())
    }
}

impl Validate for Stmt {
    fn validate(&self, valid_state: &mut PrgmValidState) -> anyhow::Result<()> {
        match self {
            Self::ParentType(name, parent_type) => {
                if valid_state.dtypes.insert(name.clone()) == false {
                    return Err(anyhow::anyhow!(
                        "Can't have two defined types named {name}"
                    ));
                }
                parent_type.validate(valid_state)?;
                return Ok(());
            }
            Self::Table(table) => {
                if valid_state.tables.insert(table.table_name.clone()) == false
                {
                    return Err(anyhow::anyhow!(
                        "Can't have two tables named {}",
                        table.table_name
                    ));
                }
                table.validate(valid_state)?;
                return Ok(());
            }
        }
    }
}

impl Validate for Prgm {
    fn validate(&self, valid_state: &mut PrgmValidState) -> anyhow::Result<()> {
        for stmt in &self.stmts {
            stmt.validate(valid_state)?;
        }
        Ok(())
    }
}

pub fn validate_prgm(prgm: &Prgm) -> anyhow::Result<()> {
    let mut valid_state = PrgmValidState {
        dtypes: HashSet::new(),
        tables: HashSet::new(),
    };
    prgm.validate(&mut valid_state)
}
