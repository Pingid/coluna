mod coerce;
mod column;
mod pipeline;
mod row;
mod schema;
mod types;

use std::collections::HashMap;

pub use coerce::*;
pub use column::*;
pub use pipeline::*;
pub use row::*;
pub use schema::*;
pub use types::*;

#[derive(Debug, Clone)]
pub enum SourceValue {
    Null,
    Boolean(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Timestamp(TemporalNano),
    Duration(DurationNano),
    Decimal128(i128),
    Utf8(String),
    LargeUtf8(String),
    Binary(Vec<u8>),
    LargeBinary(Vec<u8>),
    List(Vec<SourceValue>),
    Struct(HashMap<String, SourceValue>),
}
