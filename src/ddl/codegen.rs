use super::core::*;
use std::fmt::Display;

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Dbl(v) => write!(f, "{v}"),
            Self::Str(v) => write!(f, "{v}"),
        }
    }
}

impl<T: Display> Display for Range<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<")?;
        if let Some(min) = &self.min {
            min.fmt(f)?;
        }
        write!(f, ",")?;
        if let Some(max) = &self.max {
            max.fmt(f)?;
        }
        write!(f, ">")
    }
}

impl Display for ParentTypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParentTypeExpr::Int(range) => {
                if range.min.is_none() && range.max.is_none() {
                    return write!(f, "int");
                }
                write!(f, "int{}", range)
            }
            ParentTypeExpr::Dbl(range) => {
                if range.min.is_none() && range.max.is_none() {
                    return write!(f, "dbl");
                }
                write!(f, "dbl{}", range)
            }
            ParentTypeExpr::Str(range) => {
                if range.min.is_none() && range.max.is_none() {
                    return write!(f, "str");
                }
                write!(f, "str{}", range,)
            }
            ParentTypeExpr::Ident(val) => write!(f, "{}", val),
        }
    }
}

impl Display for DTypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.parent, if self.nullable { "?" } else { "" })
    }
}

impl Display for ColumnSchemaExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.default_value {
            Some(val) => {
                write!(f, "'{}': {} = {}", self.column_name, self.dtype, val)
            }
            None => write!(f, "'{}': {}", self.column_name, self.dtype),
        }
    }
}

impl Display for TableSchemaExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'(", self.table_name)?;
        for column in &self.columns {
            write!(f, "{}, ", column)?;
        }
        write!(f, ")")
    }
}
