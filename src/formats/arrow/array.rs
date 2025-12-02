use std::sync::Arc;

use arrow_lib::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, Date32Array, Date64Array, Decimal128Array,
    DurationMicrosecondArray, DurationMillisecondArray, DurationNanosecondArray,
    DurationSecondArray, FixedSizeBinaryArray, Float16Array, Float32Array, Float64Array, Int8Array,
    Int16Array, Int32Array, Int64Array, LargeBinaryArray, LargeStringArray, ListArray, NullArray,
    StringArray, StructArray, Time32MillisecondArray, Time32SecondArray, Time64MicrosecondArray,
    Time64NanosecondArray, TimestampMicrosecondArray, TimestampMillisecondArray,
    TimestampNanosecondArray, TimestampSecondArray, UInt8Array, UInt16Array, UInt32Array,
    UInt64Array,
};
use arrow_lib::buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_lib::datatypes::{ArrowNativeType, DataType, Field, TimeUnit};
use arrow_lib::error::ArrowError;

use crate::encoding::{ColumnData, ColumnValues, TemporalNano};

pub fn column_to_array(field: &Field, buf: ColumnData) -> Result<ArrayRef, ArrowError> {
    use {ColumnValues as BK, DataType as DT};

    let ColumnData { kind, validity } = buf;

    fn make_nulls(validity: Vec<bool>) -> Option<NullBuffer> {
        if validity.iter().all(|&v| v) {
            None
        } else {
            Some(NullBuffer::from(validity))
        }
    }

    fn scalar<R: Array, I: IntoIterator<Item = A>, A: Into<B>, B: ArrowNativeType>(
        f: impl FnOnce(ScalarBuffer<B>, Option<NullBuffer>) -> R, values: I, validity: Vec<bool>,
    ) -> Arc<R> {
        let v: ScalarBuffer<B> = values.into_iter().map(|v| v.into()).collect();
        Arc::new(f(v, make_nulls(validity)))
    }

    let arr: ArrayRef = match (field.data_type(), kind) {
        // Null
        (DT::Null, BK::Null) => Arc::new(NullArray::new(validity.len())),

        // Boolean
        (DT::Boolean, BK::Boolean(v)) => {
            Arc::new(BooleanArray::new(v.into(), make_nulls(validity)))
        }

        // Signed ints
        (DT::Int8, BK::Int8(v)) => scalar(Int8Array::new, v, validity),
        (DT::Int16, BK::Int16(v)) => scalar(Int16Array::new, v, validity),
        (DT::Int32, BK::Int32(v)) => scalar(Int32Array::new, v, validity),
        (DT::Int64, BK::Int64(v)) => scalar(Int64Array::new, v, validity),

        // Unsigned ints
        (DT::UInt8, BK::UInt8(v)) => scalar(UInt8Array::new, v, validity),
        (DT::UInt16, BK::UInt16(v)) => scalar(UInt16Array::new, v, validity),
        (DT::UInt32, BK::UInt32(v)) => scalar(UInt32Array::new, v, validity),
        (DT::UInt64, BK::UInt64(v)) => scalar(UInt64Array::new, v, validity),

        // Floats
        (DT::Float16, BK::Float32(v)) => scalar(Float16Array::new, to_f16(v), validity),
        (DT::Float32, BK::Float32(v)) => scalar(Float32Array::new, v, validity),
        (DT::Float64, BK::Float64(v)) => scalar(Float64Array::new, v, validity),

        // Timestamps
        (DT::Timestamp(unit, _tz), BK::Timestamp(v)) => {
            let v = from_nanos(unit, v);
            match unit {
                TimeUnit::Second => scalar(TimestampSecondArray::new, v, validity),
                TimeUnit::Millisecond => scalar(TimestampMillisecondArray::new, v, validity),
                TimeUnit::Microsecond => scalar(TimestampMicrosecondArray::new, v, validity),
                TimeUnit::Nanosecond => scalar(TimestampNanosecondArray::new, v, validity),
            }
        }

        // Date types
        (DT::Date32, BK::Timestamp(v)) => {
            let v = v.into_iter().map(|v| v.to_date32());
            scalar(Date32Array::new, v, validity)
        }
        (DT::Date64, BK::Timestamp(v)) => {
            let v = v.into_iter().map(|v| v.to_date64());
            scalar(Date64Array::new, v, validity)
        }

        // Time types
        (DT::Time32(TimeUnit::Second), BK::Timestamp(v)) => {
            let v = v.into_iter().map(|v| v.to_time32_s());
            scalar(Time32SecondArray::new, v, validity)
        }
        (DT::Time32(TimeUnit::Millisecond), BK::Timestamp(v)) => {
            let v = v.into_iter().map(|v| v.to_time32_ms());
            scalar(Time32MillisecondArray::new, v, validity)
        }
        (DT::Time64(TimeUnit::Microsecond), BK::Timestamp(v)) => {
            let v = v.into_iter().map(|v| v.to_time64_us());
            scalar(Time64MicrosecondArray::new, v, validity)
        }
        (DT::Time64(TimeUnit::Nanosecond), BK::Timestamp(v)) => {
            let v = v.into_iter().map(|v| v.to_time64_ns());
            scalar(Time64NanosecondArray::new, v, validity)
        }

        // Duration
        (DT::Duration(TimeUnit::Second), BK::Duration(v)) => {
            let v = v.into_iter().map(|v| v.as_secs());
            scalar(DurationSecondArray::new, v, validity)
        }
        (DT::Duration(TimeUnit::Millisecond), BK::Duration(v)) => {
            let v = v.into_iter().map(|v| v.as_millis());
            scalar(DurationMillisecondArray::new, v, validity)
        }
        (DT::Duration(TimeUnit::Microsecond), BK::Duration(v)) => {
            let v = v.into_iter().map(|v| v.as_micros());
            scalar(DurationMicrosecondArray::new, v, validity)
        }
        (DT::Duration(TimeUnit::Nanosecond), BK::Duration(v)) => {
            let v = v.into_iter().map(|v| v.as_nanos());
            scalar(DurationNanosecondArray::new, v, validity)
        }

        // Interval - not yet supported
        (DT::Interval(_), _) => {
            return Err(ArrowError::NotYetImplemented("Interval type".to_string()));
        }

        // Binary types
        (DT::Binary, BK::Binary { offsets, values }) => {
            let nulls = make_nulls(validity);
            let offsets = OffsetBuffer::new(offsets.into());
            Arc::new(BinaryArray::new(offsets, values.into(), nulls))
        }
        (
            DT::FixedSizeBinary(size),
            BK::FixedSizeBinary {
                size: buf_size,
                values,
            },
        ) => {
            assert_eq!(*size, buf_size, "FixedSizeBinary size mismatch");
            let nulls = make_nulls(validity);
            Arc::new(FixedSizeBinaryArray::new(*size, values.into(), nulls))
        }
        (DT::LargeBinary, BK::LargeBinary { offsets, values }) => {
            let nulls = make_nulls(validity);
            let offsets = OffsetBuffer::new(offsets.into());
            Arc::new(LargeBinaryArray::new(offsets, values.into(), nulls))
        }
        (DT::BinaryView, _) => {
            return Err(ArrowError::NotYetImplemented("BinaryView type".to_string()));
        }

        // Utf8 strings
        (DT::Utf8, BK::Utf8 { offsets, values }) => {
            let nulls = make_nulls(validity);
            let offsets = OffsetBuffer::new(offsets.into());
            Arc::new(StringArray::new(offsets, values.into(), nulls))
        }
        (DT::LargeUtf8, BK::LargeUtf8 { offsets, values }) => {
            let nulls = make_nulls(validity);
            let offsets = OffsetBuffer::new(offsets.into());
            Arc::new(LargeStringArray::new(offsets, values.into(), nulls))
        }
        (DT::Utf8View, _) => {
            return Err(ArrowError::NotYetImplemented("Utf8View type".to_string()));
        }

        // List
        (DT::List(list_field), BK::List { offsets, values, .. }) => {
            let nulls = make_nulls(validity);
            let offsets = OffsetBuffer::new(offsets.into());
            let child_array = column_to_array(list_field, *values)?;
            Arc::new(ListArray::new(
                list_field.clone(),
                offsets,
                child_array,
                nulls,
            ))
        }
        (DT::ListView(_), _) => {
            return Err(ArrowError::NotYetImplemented("ListView type".to_string()));
        }
        (DT::FixedSizeList(_, _), _) => {
            return Err(ArrowError::NotYetImplemented(
                "FixedSizeList type".to_string(),
            ));
        }
        (DT::LargeList(_), _) => {
            return Err(ArrowError::NotYetImplemented("LargeList type".to_string()));
        }
        (DT::LargeListView(_), _) => {
            return Err(ArrowError::NotYetImplemented(
                "LargeListView type".to_string(),
            ));
        }

        // Struct
        (
            DT::Struct(fields),
            BK::Struct {
                fields: field_outputs,
            },
        ) => {
            let nulls = make_nulls(validity);
            let mut child_arrays = Vec::with_capacity(fields.len());
            for (field, (_col_field, output)) in fields.iter().zip(field_outputs.into_iter()) {
                child_arrays.push(column_to_array(field, output)?);
            }
            Arc::new(StructArray::new(fields.clone(), child_arrays, nulls))
        }

        // Union - not yet supported
        (DT::Union(_, _), _) => {
            return Err(ArrowError::NotYetImplemented("Union type".to_string()));
        }

        // Dictionary - not yet supported
        (DT::Dictionary(_, _), _) => {
            return Err(ArrowError::NotYetImplemented("Dictionary type".to_string()));
        }

        // Decimal types
        #[cfg(not(feature = "arrow-54"))]
        (DT::Decimal32(_, _), _) => {
            return Err(ArrowError::NotYetImplemented("Decimal32 type".to_string()));
        }
        #[cfg(not(feature = "arrow-54"))]
        (DT::Decimal64(_, _), _) => {
            return Err(ArrowError::NotYetImplemented("Decimal64 type".to_string()));
        }
        (DT::Decimal128(precision, scale), BK::Decimal128(v)) => {
            let nulls = make_nulls(validity);
            Arc::new(
                Decimal128Array::new(v.into(), nulls)
                    .with_precision_and_scale(*precision, *scale)?,
            )
        }
        (DT::Decimal256(_, _), _) => {
            return Err(ArrowError::NotYetImplemented("Decimal256 type".to_string()));
        }

        // Map - not yet supported
        (DT::Map(_, _), _) => {
            return Err(ArrowError::NotYetImplemented("Map type".to_string()));
        }

        // RunEndEncoded - not yet supported
        (DT::RunEndEncoded(_, _), _) => {
            return Err(ArrowError::NotYetImplemented(
                "RunEndEncoded type".to_string(),
            ));
        }

        // Fallback for mismatched type/buffer combinations
        (dt, _kind) => {
            return Err(ArrowError::InvalidArgumentError(format!(
                "Unsupported data type / buffer combination: {:?}",
                dt,
            )));
        }
    };

    Ok(arr)
}

/// ---------------- Utility functions ----------------
fn to_f16(v: impl IntoIterator<Item = f32>) -> Vec<half::f16> {
    v.into_iter().map(|v| half::f16::from_f32(v)).collect()
}

fn from_nanos(unit: &TimeUnit, v: impl IntoIterator<Item = TemporalNano>) -> Vec<i64> {
    match unit {
        TimeUnit::Second => v.into_iter().map(|v| v.as_secs()).collect(),
        TimeUnit::Millisecond => v.into_iter().map(|v| v.as_millis()).collect(),
        TimeUnit::Microsecond => v.into_iter().map(|v| v.as_micros()).collect(),
        TimeUnit::Nanosecond => v.into_iter().map(|v| v.as_nanos()).collect(),
    }
}
