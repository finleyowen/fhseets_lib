pub mod codegen;
pub mod json;

use anyhow::Ok;

use crate::{json::ToJson, ql::lex::Literal};
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

    fn validate_literal(&self, lit: Option<&Literal>) -> anyhow::Result<()> {
        match lit {
            Some(lit) => self.validator(lit),
            None => {
                if self.get_nullable() {
                    return Ok(());
                } else {
                    return Err(anyhow::anyhow!("Required value was null!"));
                }
            }
        }
    }

    fn validate_data_type(&self) -> anyhow::Result<()>;

    fn validator(&self, lit: &Literal) -> anyhow::Result<()>;
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

    fn validate_i32(&self, val: i32) -> anyhow::Result<()> {
        if let Some(min) = self.min
            && val < min
        {
            return Err(anyhow::anyhow!(
                "Minimum value {} (entered {})",
                min,
                val
            ));
        }

        if let Some(max) = self.max
            && val > max
        {
            return Err(anyhow::anyhow!(
                "Maximum value {} (entered {})",
                max,
                val
            ));
        }

        Ok(())
    }
}

impl DataType for IntDataType {
    fn get_nullable(&self) -> bool {
        self.nullable
    }

    fn validator(&self, lit: &Literal) -> anyhow::Result<()> {
        match lit {
            Literal::Int(val) => self.validate_i32(*val),
            _ => Err(anyhow::anyhow!(
                "Couldn't validate non-integer literal against integer type."
            )),
        }
    }

    fn validate_data_type(&self) -> anyhow::Result<()> {
        if let Some(min) = self.min
            && let Some(max) = self.max
            && min > max
        {
            return Err(anyhow::anyhow!(
                "Can't have min ({min}) > max ({max})"
            ));
        }
        Ok(())
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

    fn validate_f64(&self, val: f64) -> anyhow::Result<()> {
        if let Some(min) = self.min
            && val < min
        {
            return Err(anyhow::anyhow!(
                "Minimum value {} (entered {})",
                min,
                val
            ));
        }

        if let Some(max) = self.max
            && val > max
        {
            return Err(anyhow::anyhow!(
                "Maximum value {} (entered {})",
                max,
                val
            ));
        }

        Ok(())
    }
}

impl DataType for DblDataType {
    fn get_nullable(&self) -> bool {
        self.nullable
    }

    fn validator(&self, lit: &Literal) -> anyhow::Result<()> {
        match lit {
            Literal::Dbl(val) => self.validate_f64(*val),
            _ => Err(anyhow::anyhow!(
                "Couldn't validate non-double literal against double type."
            )),
        }
    }

    fn validate_data_type(&self) -> anyhow::Result<()> {
        if let Some(min) = self.min
            && let Some(max) = self.max
            && min > max
        {
            return Err(anyhow::anyhow!(
                "Can't have min ({min}) > max ({max})"
            ));
        }
        Ok(())
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

    fn validate_str(&self, s: &str) -> anyhow::Result<()> {
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

impl DataType for StrDataType {
    fn get_nullable(&self) -> bool {
        self.nullable
    }

    fn validator(&self, lit: &Literal) -> anyhow::Result<()> {
        match lit {
            Literal::Str(val) => self.validate_str(&val),
            _ => Err(anyhow::anyhow!(
                "Couldn't validate non-string literal against string type."
            )),
        }
    }

    fn validate_data_type(&self) -> anyhow::Result<()> {
        if let Some(min) = self.min
            && let Some(max) = self.max
            && min > max
        {
            return Err(anyhow::anyhow!(
                "Can't have min ({min}) > max ({max})"
            ));
        }
        Ok(())
    }
}

/// Represents a column schema in the application.
pub struct ColumnSchema {
    column_name: Rc<str>,
    column_type: Rc<dyn DataType>,
    default_value: Option<Literal>,
}

impl ColumnSchema {
    pub fn new(
        column_name: Rc<str>,
        column_type: Rc<dyn DataType>,
        default_value: Option<Literal>,
    ) -> Self {
        Self {
            column_name,
            column_type,
            default_value,
        }
    }

    pub fn get_name(&self) -> Rc<str> {
        self.column_name.clone()
    }

    pub fn get_type(&self) -> Rc<dyn DataType> {
        self.column_type.clone()
    }

    pub fn validate_column_schema(&self) -> anyhow::Result<()> {
        self.get_type().validate_data_type()
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

    pub fn validate_table_schema(&self) -> anyhow::Result<()> {
        for col in &self.columns {
            col.validate_column_schema()?;
        }
        Ok(())
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

    pub fn validate_spreadsheet_schema(&self) -> anyhow::Result<()> {
        for table in &self.tables {
            table.validate_table_schema()?;
        }
        Ok(())
    }
}
