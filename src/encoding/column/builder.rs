use crate::encoding::{
    CoerceError, CoercePolicy, ColumnData, DurationNano, SourceValue, TemporalNano,
};

mod nested;
mod nullable;
mod storage;

pub use nested::{ListBuilder, StructBuilder};
pub use nullable::Builder;
pub use storage::{
    BinaryStorage, FinishableStorage, FixedSizeBinaryStorage, LargeBinaryStorage, LargeUtf8Storage,
    NullStorage, PrimitiveStorage, Utf8Storage, ValueStorage,
};

#[derive(Debug, Clone)]
pub enum ColumnBuilder {
    // leaf / primitive
    Null(Builder<NullStorage>),
    Boolean(Builder<PrimitiveStorage<bool>>),
    Int8(Builder<PrimitiveStorage<i8>>),
    Int16(Builder<PrimitiveStorage<i16>>),
    Int32(Builder<PrimitiveStorage<i32>>),
    Int64(Builder<PrimitiveStorage<i64>>),
    UInt8(Builder<PrimitiveStorage<u8>>),
    UInt16(Builder<PrimitiveStorage<u16>>),
    UInt32(Builder<PrimitiveStorage<u32>>),
    UInt64(Builder<PrimitiveStorage<u64>>),
    Float32(Builder<PrimitiveStorage<f32>>),
    Float64(Builder<PrimitiveStorage<f64>>),
    Timestamp(Builder<PrimitiveStorage<TemporalNano>>),
    Duration(Builder<PrimitiveStorage<DurationNano>>),
    Decimal128(Builder<PrimitiveStorage<i128>>),
    Utf8(Builder<Utf8Storage>),
    LargeUtf8(Builder<LargeUtf8Storage>),
    Binary(Builder<BinaryStorage>),
    LargeBinary(Builder<LargeBinaryStorage>),
    FixedSizeBinary(Builder<FixedSizeBinaryStorage>),

    // nested
    List(ListBuilder),
    Struct(StructBuilder),
}

impl ColumnBuilder {
    pub fn len(&self) -> usize {
        use ColumnBuilder::*;
        match self {
            Null(b) => b.len(),
            Boolean(b) => b.len(),
            Int8(b) => b.len(),
            Int16(b) => b.len(),
            Int32(b) => b.len(),
            Int64(b) => b.len(),
            UInt8(b) => b.len(),
            UInt16(b) => b.len(),
            UInt32(b) => b.len(),
            UInt64(b) => b.len(),
            Float32(b) => b.len(),
            Float64(b) => b.len(),
            Timestamp(b) => b.len(),
            Duration(b) => b.len(),
            Decimal128(b) => b.len(),
            Utf8(b) => b.len(),
            LargeUtf8(b) => b.len(),
            Binary(b) => b.len(),
            LargeBinary(b) => b.len(),
            FixedSizeBinary(b) => b.len(),
            List(b) => b.len(),
            Struct(b) => b.len(),
        }
    }

    /// Recursively appends a null/default value to keep lengths in sync
    pub fn append_null(&mut self) {
        use ColumnBuilder::*;
        match self {
            // Primitives: delegate to the NullableBuilder's append_null
            Null(b) => b.append_null(),
            Boolean(b) => b.append_null(),
            Int8(b) => b.append_null(),
            Int16(b) => b.append_null(),
            Int32(b) => b.append_null(),
            Int64(b) => b.append_null(),
            UInt8(b) => b.append_null(),
            UInt16(b) => b.append_null(),
            UInt32(b) => b.append_null(),
            UInt64(b) => b.append_null(),
            Float32(b) => b.append_null(),
            Float64(b) => b.append_null(),
            Timestamp(b) => b.append_null(),
            Duration(b) => b.append_null(),
            Decimal128(b) => b.append_null(),
            Utf8(b) => b.append_null(),
            LargeUtf8(b) => b.append_null(),
            Binary(b) => b.append_null(),
            LargeBinary(b) => b.append_null(),
            FixedSizeBinary(b) => b.append_null(),
            // Nested: Recursive step
            List(b) => b.append_null(),
            Struct(b) => b.append_null(),
        }
    }

    pub fn append_raw<P: CoercePolicy>(
        &mut self, policy: &P, raw: SourceValue,
    ) -> Result<(), CoerceError> {
        use ColumnBuilder::*;

        match self {
            Null(b) => {
                let v = policy.coerce_null(raw)?;
                b.append(v);
                Ok(())
            }
            Boolean(b) => {
                let v = policy.coerce_bool(raw)?;
                b.append(v);
                Ok(())
            }
            Int8(b) => {
                let v = policy.coerce_i8(raw)?;
                b.append(v);
                Ok(())
            }
            Int16(b) => {
                let v = policy.coerce_i16(raw)?;
                b.append(v);
                Ok(())
            }
            Int32(b) => {
                let v = policy.coerce_i32(raw)?;
                b.append(v);
                Ok(())
            }
            Int64(b) => {
                let v = policy.coerce_i64(raw)?;
                b.append(v);
                Ok(())
            }
            UInt8(b) => {
                let v = policy.coerce_u8(raw)?;
                b.append(v);
                Ok(())
            }
            UInt16(b) => {
                let v = policy.coerce_u16(raw)?;
                b.append(v);
                Ok(())
            }
            UInt32(b) => {
                let v = policy.coerce_u32(raw)?;
                b.append(v);
                Ok(())
            }
            UInt64(b) => {
                let v = policy.coerce_u64(raw)?;
                b.append(v);
                Ok(())
            }
            Float32(b) => {
                let v = policy.coerce_f32(raw)?;
                b.append(v);
                Ok(())
            }
            Float64(b) => {
                let v = policy.coerce_f64(raw)?;
                b.append(v);
                Ok(())
            }
            Timestamp(b) => {
                let v = policy.coerce_timestamp(raw)?;
                b.append(v);
                Ok(())
            }
            Duration(b) => {
                let v = policy.coerce_duration(raw)?;
                b.append(v);
                Ok(())
            }
            Decimal128(b) => {
                let v = policy.coerce_decimal128(raw)?;
                b.append(v);
                Ok(())
            }
            Utf8(b) => {
                let s = policy.coerce_str(raw)?;
                b.append(s.map(|s| s.into_bytes()));
                Ok(())
            }
            LargeUtf8(b) => {
                let s = policy.coerce_str(raw)?;
                b.append(s.map(|s| s.into_bytes()));
                Ok(())
            }
            Binary(b) => {
                let s = policy.coerce_binary(raw)?;
                b.append(s);
                Ok(())
            }
            LargeBinary(b) => {
                let s = policy.coerce_binary(raw)?;
                b.append(s);
                Ok(())
            }
            FixedSizeBinary(b) => {
                let s = policy.coerce_binary(raw)?;
                b.append(s);
                Ok(())
            }

            // nested cols delegate to nested builders
            List(b) => b.append_raw(policy, raw),
            Struct(b) => b.append_raw(policy, raw),
        }
    }

    pub fn finish(&mut self) -> ColumnData {
        use ColumnBuilder::*;
        match self {
            Null(b) => b.finish(),
            Boolean(b) => b.finish(),
            Int8(b) => b.finish(),
            Int16(b) => b.finish(),
            Int32(b) => b.finish(),
            Int64(b) => b.finish(),
            UInt8(b) => b.finish(),
            UInt16(b) => b.finish(),
            UInt32(b) => b.finish(),
            UInt64(b) => b.finish(),
            Float32(b) => b.finish(),
            Float64(b) => b.finish(),
            Timestamp(b) => b.finish(),
            Duration(b) => b.finish(),
            Decimal128(b) => b.finish(),
            Utf8(b) => b.finish(),
            LargeUtf8(b) => b.finish(),
            Binary(b) => b.finish(),
            LargeBinary(b) => b.finish(),
            FixedSizeBinary(b) => b.finish(),
            List(b) => b.finish(),
            Struct(b) => b.finish(),
        }
    }
}
