use std::sync::Arc;

use crate::encoding::{DurationNano, Field, TemporalNano};

mod builder;
pub use builder::*;

pub trait ColumnSpec {
    fn build_columns(&self, rows: usize, value_hint: usize) -> Vec<ColumnBinding>;
}

impl<T> ColumnSpec for Arc<T>
where T: ColumnSpec
{
    fn build_columns(&self, rows: usize, value_hint: usize) -> Vec<ColumnBinding> {
        (**self).build_columns(rows, value_hint)
    }
}

#[derive(Debug, Clone)]
pub struct ColumnBinding {
    pub field: Field,
    pub builder: ColumnBuilder,
}

impl ColumnBinding {
    pub fn new(field: Field, builder: ColumnBuilder) -> Self {
        Self { field, builder }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnData {
    pub kind: ColumnValues,
    pub validity: Vec<bool>,
}

#[derive(Debug, Clone)]
pub enum ColumnValues {
    Null,
    Boolean(Vec<bool>),
    Int8(Vec<i8>),
    Int16(Vec<i16>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    UInt32(Vec<u32>),
    UInt64(Vec<u64>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Timestamp(Vec<TemporalNano>),
    Duration(Vec<DurationNano>),
    Decimal128(Vec<i128>),
    Utf8 {
        offsets: Vec<i32>,
        values: Vec<u8>,
    },
    LargeUtf8 {
        offsets: Vec<i64>,
        values: Vec<u8>,
    },
    Binary {
        offsets: Vec<i32>,
        values: Vec<u8>,
    },
    LargeBinary {
        offsets: Vec<i64>,
        values: Vec<u8>,
    },
    FixedSizeBinary {
        size: i32,
        values: Vec<u8>,
    },
    List {
        field: Field,
        offsets: Vec<i32>,
        values: Box<ColumnData>,
    },
    Struct {
        fields: Vec<(Field, ColumnData)>,
    },
}

impl ColumnData {
    pub fn len(&self) -> usize {
        match &self.kind {
            ColumnValues::Null => self.validity.len(),
            ColumnValues::Boolean(v) => v.len(),
            ColumnValues::Int8(v) => v.len(),
            ColumnValues::Int16(v) => v.len(),
            ColumnValues::Int32(v) => v.len(),
            ColumnValues::Int64(v) => v.len(),
            ColumnValues::UInt8(v) => v.len(),
            ColumnValues::UInt16(v) => v.len(),
            ColumnValues::UInt32(v) => v.len(),
            ColumnValues::UInt64(v) => v.len(),
            ColumnValues::Float32(v) => v.len(),
            ColumnValues::Float64(v) => v.len(),
            ColumnValues::Timestamp(v) => v.len(),
            ColumnValues::Duration(v) => v.len(),
            ColumnValues::Decimal128(v) => v.len(),
            ColumnValues::Utf8 { offsets, .. } => offsets.len().saturating_sub(1),
            ColumnValues::LargeUtf8 { offsets, .. } => offsets.len().saturating_sub(1),
            ColumnValues::Binary { offsets, .. } => offsets.len().saturating_sub(1),
            ColumnValues::LargeBinary { offsets, .. } => offsets.len().saturating_sub(1),
            ColumnValues::FixedSizeBinary { size, values } => {
                if *size == 0 {
                    0
                } else {
                    values.len() / (*size as usize)
                }
            }
            ColumnValues::List { offsets, .. } => offsets.len().saturating_sub(1),
            ColumnValues::Struct { fields } => {
                fields.first().map(|(_, data)| data.len()).unwrap_or(0)
            }
        }
    }
}
