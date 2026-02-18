pub mod codegen;
pub mod json;

use crate::json::ToJson;
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

pub const INT_TYPE_NAME: &str = "int";
pub const DBL_TYPE_NAME: &str = "dbl";
pub const STR_TYPE_NAME: &str = "str";

/// Represents a data type in the application.
pub trait DataType: ToJson + Display {
    fn get_nullable(&self) -> bool;

    fn validator(&self, s: &str) -> anyhow::Result<()>;

    fn validate(&self, s: &str) -> anyhow::Result<()> {
        if self.get_nullable() && s.is_empty() {
            return Ok(());
        }
        self.validator(s)
    }
}

/// Type alias over `Rc<dyn DataType>` for convenience.
pub type SharedDataType = Rc<dyn DataType>;

/// Represents an integer data type in the application.
#[derive(Debug)]
pub struct IntDataType {
    min: Option<i32>,
    max: Option<i32>,
    nullable: bool,
}

impl IntDataType {
    pub fn new(min: Option<i32>, max: Option<i32>, nullable: bool) -> Self {
        Self { min, max, nullable }
    }
}

impl DataType for IntDataType {
    fn get_nullable(&self) -> bool {
        self.nullable
    }

    fn validator(&self, s: &str) -> anyhow::Result<()> {
        match s.parse::<i32>() {
            Ok(val) => {
                if let Some(min) = &self.min
                    && *min > val
                {
                    return Err(anyhow::anyhow!("Minimum value {min}"));
                }
                if let Some(max) = &self.max
                    && *max < val
                {
                    return Err(anyhow::anyhow!("Maximum value {max}"));
                }
                Ok(())
            }
            Err(_) => Err(anyhow::anyhow!("Couldn't parse int from {}", s)),
        }
    }
}

/// Represents a double data type in the application.
#[derive(Debug)]

pub struct DblDataType {
    min: Option<f64>,
    max: Option<f64>,
    nullable: bool,
}

impl DblDataType {
    pub fn new(min: Option<f64>, max: Option<f64>, nullable: bool) -> Self {
        Self { min, max, nullable }
    }
}

impl DataType for DblDataType {
    fn get_nullable(&self) -> bool {
        self.nullable
    }

    fn validator(&self, s: &str) -> anyhow::Result<()> {
        match s.parse::<f64>() {
            Ok(val) => {
                if let Some(min) = &self.min
                    && *min > val
                {
                    return Err(anyhow::anyhow!("Minimum value {min}"));
                }
                if let Some(max) = &self.max
                    && *max < val
                {
                    return Err(anyhow::anyhow!("Maximum value {max}"));
                }
                Ok(())
            }
            Err(_) => Err(anyhow::anyhow!("Couldn't parse int from {}", s)),
        }
    }
}

/// Represents a string data type in the application.
#[derive(Debug)]
pub struct StrDataType {
    min: Option<usize>,
    max: Option<usize>,
    nullable: bool,
}

impl StrDataType {
    pub fn new(min: Option<usize>, max: Option<usize>, nullable: bool) -> Self {
        Self { min, max, nullable }
    }
}

impl DataType for StrDataType {
    fn get_nullable(&self) -> bool {
        self.nullable
    }

    fn validator(&self, s: &str) -> anyhow::Result<()> {
        if let Some(min) = &self.min
            && s.len() < *min
        {
            return Err(anyhow::anyhow!("Minimum length {min}"));
        }
        if let Some(max) = &self.max
            && s.len() < *max
        {
            return Err(anyhow::anyhow!("Minimum length {max}"));
        }
        Ok(())
    }
}

/// Represents a column schema in the application.
pub struct ColumnSchema {
    column_name: Rc<str>,
    column_type: Rc<dyn DataType>,
}

impl ColumnSchema {
    pub fn new(column_name: Rc<str>, column_type: Rc<dyn DataType>) -> Self {
        Self {
            column_name,
            column_type,
        }
    }

    pub fn get_name(&self) -> Rc<str> {
        self.column_name.clone()
    }

    pub fn get_type(&self) -> Rc<dyn DataType> {
        self.column_type.clone()
    }
}

/// Represents a table schema in the application.
pub struct TableSchema {
    table_name: Rc<str>,
    columns: Vec<ColumnSchema>,
}

impl TableSchema {
    pub fn new(table_name: Rc<str>, columns: Vec<ColumnSchema>) -> Self {
        Self {
            table_name,
            columns,
        }
    }

    pub fn get_name(&self) -> Rc<str> {
        self.table_name.clone()
    }

    pub fn get_num_columns(&self) -> usize {
        self.columns.len()
    }

    pub fn get_column(&self, idx: usize) -> Option<&ColumnSchema> {
        self.columns.get(idx)
    }
}

pub type SharedTableSchema = Rc<TableSchema>;

/// Represents a database schema in the application.
pub struct SpreadsheetSchema {
    ss_name: Rc<str>,
    tables: Vec<Rc<TableSchema>>,
}

impl SpreadsheetSchema {
    pub fn new(ss_name: Rc<str>, tables: Vec<Rc<TableSchema>>) -> Self {
        Self { ss_name, tables }
    }

    pub fn get_name(&self) -> Rc<str> {
        self.ss_name.clone()
    }

    pub fn get_num_tables(&self) -> usize {
        self.tables.len()
    }

    pub fn get_table(&self, idx: usize) -> Option<&Rc<TableSchema>> {
        self.tables.get(idx)
    }
}
