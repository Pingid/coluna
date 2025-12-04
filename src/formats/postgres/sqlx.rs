use postgres_types::Type;
use sqlx::postgres::{PgColumn, PgRow};
use sqlx::{Column, Row};

use crate::encoding::{
    Builder, ColumnBinding, ColumnBuilder, ColumnSpec, DurationNano, Field, FinishableStorage,
    RowAccess, SourceValue, TemporalNano, TypeId, ValueStorage,
};

#[derive(Debug, thiserror::Error)]
pub enum PgRowError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl RowAccess for PgRow {
    type Error = PgRowError;

    fn get_boolean(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self
            .try_get::<Option<bool>, _>(idx)?
            .map(SourceValue::Boolean))
    }

    fn get_int8(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self.try_get::<Option<i8>, _>(idx)?.map(SourceValue::Int8))
    }

    fn get_int16(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self.try_get::<Option<i16>, _>(idx)?.map(SourceValue::Int16))
    }

    fn get_int32(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self.try_get::<Option<i32>, _>(idx)?.map(SourceValue::Int32))
    }

    fn get_int64(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self.try_get::<Option<i64>, _>(idx)?.map(SourceValue::Int64))
    }

    fn get_uint32(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        // PostgreSQL OID type - decode as Oid wrapper and extract u32
        if let Ok(Some(oid)) = self.try_get::<Option<sqlx::postgres::types::Oid>, _>(idx) {
            return Ok(Some(SourceValue::UInt32(oid.0)));
        }
        // Fallback: decode as i32 and cast
        Ok(self
            .try_get::<Option<i32>, _>(idx)?
            .map(|v| SourceValue::UInt32(v as u32)))
    }

    fn get_float32(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self
            .try_get::<Option<f32>, _>(idx)?
            .map(SourceValue::Float32))
    }

    fn get_float64(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self
            .try_get::<Option<f64>, _>(idx)?
            .map(SourceValue::Float64))
    }

    fn get_decimal128(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        // PostgreSQL NUMERIC - decode via sqlx's Decimal type (rust_decimal)
        use sqlx::types::Decimal;
        let dec: Option<Decimal> = self.try_get(idx)?;
        // Scale to i128 with 18 decimal places
        Ok(dec.map(|d| {
            let scaled = d * Decimal::from(1_000_000_000_000_000_000i64);
            // mantissa() returns the raw i128 value
            SourceValue::Decimal128(scaled.mantissa())
        }))
    }

    fn get_timestamp(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        // Try TIMESTAMPTZ first, then TIMESTAMP, then DATE
        if let Ok(Some(dt)) = self.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(idx) {
            return Ok(Some(SourceValue::Timestamp(TemporalNano::from_millis(
                dt.timestamp_millis(),
            ))));
        }
        if let Ok(Some(dt)) = self.try_get::<Option<chrono::NaiveDateTime>, _>(idx) {
            return Ok(Some(SourceValue::Timestamp(TemporalNano::from_millis(
                dt.and_utc().timestamp_millis(),
            ))));
        }
        if let Ok(Some(d)) = self.try_get::<Option<chrono::NaiveDate>, _>(idx) {
            let dt = d.and_hms_opt(0, 0, 0).unwrap();
            return Ok(Some(SourceValue::Timestamp(TemporalNano::from_millis(
                dt.and_utc().timestamp_millis(),
            ))));
        }
        if let Ok(Some(t)) = self.try_get::<Option<chrono::NaiveTime>, _>(idx) {
            let nanos =
                t.num_seconds_from_midnight() as i64 * 1_000_000_000 + t.nanosecond() as i64;
            return Ok(Some(SourceValue::Timestamp(TemporalNano(nanos))));
        }
        Ok(None)
    }

    fn get_duration(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        // PostgreSQL INTERVAL - sqlx decodes to PgInterval
        if let Ok(Some(interval)) =
            self.try_get::<Option<sqlx::postgres::types::PgInterval>, _>(idx)
        {
            // PgInterval has months, days, microseconds
            // Convert to nanoseconds (approximating months as 30 days)
            let total_days = interval.months as i64 * 30 + interval.days as i64;
            let nanos = total_days * 86_400_000_000_000 + interval.microseconds * 1_000;
            return Ok(Some(SourceValue::Duration(DurationNano(nanos))));
        }
        Ok(None)
    }

    fn get_utf8(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        // Try String first
        if let Ok(s) = self.try_get::<Option<String>, _>(idx) {
            return Ok(s.map(SourceValue::Utf8));
        }
        // MONEY type - decode as PgMoney and format
        if let Ok(Some(m)) = self.try_get::<Option<sqlx::postgres::types::PgMoney>, _>(idx) {
            return Ok(Some(SourceValue::Utf8(format!("{}", m.0))));
        }
        // Fallback: return None for unsupported types
        Ok(None)
    }

    fn get_large_utf8(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self
            .try_get::<Option<String>, _>(idx)?
            .map(SourceValue::LargeUtf8))
    }

    fn get_binary(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        // Try BYTEA - BIT/VARBIT types are not directly supported, return None for those
        match self.try_get::<Option<Vec<u8>>, _>(idx) {
            Ok(b) => Ok(b.map(SourceValue::Binary)),
            Err(_) => Ok(None), // BIT types etc. - not supported
        }
    }

    fn get_large_binary(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self
            .try_get::<Option<Vec<u8>>, _>(idx)?
            .map(SourceValue::LargeBinary))
    }

    fn get_fixed_size_binary(
        &mut self, _col: &ColumnBinding, idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        Ok(self
            .try_get::<Option<Vec<u8>>, _>(idx)?
            .map(SourceValue::Binary))
    }
}

use chrono::Timelike;

impl ColumnSpec for &[PgColumn] {
    fn build_columns(&self, rows: usize, value_hint: usize) -> Vec<ColumnBinding> {
        self.iter()
            .map(|col| {
                let oid = col.type_info().oid().map(|o| o.0);
                let pg_type = oid.and_then(Type::from_oid);
                let (type_id, builder) = get_type_builder(pg_type, rows, value_hint);
                ColumnBinding::new(
                    Field::new(col.name().to_string(), type_id, true), // assume nullable
                    builder,
                )
            })
            .collect()
    }
}

fn get_type_builder(tp: Option<Type>, rows: usize, vals: usize) -> (TypeId, ColumnBuilder) {
    use {ColumnBuilder as B, TypeId as TI};

    let Some(tp) = tp else {
        return (TI::Utf8, B::Utf8(builder(rows, vals)));
    };

    match tp {
        Type::BOOL => (TI::Boolean, B::Boolean(builder(rows, vals))),
        Type::INT2 => (TI::Int16, B::Int16(builder(rows, vals))),
        Type::INT4 => (TI::Int32, B::Int32(builder(rows, vals))),
        Type::INT8 => (TI::Int64, B::Int64(builder(rows, vals))),
        Type::OID => (TI::UInt32, B::UInt32(builder(rows, vals))),
        Type::FLOAT4 => (TI::Float32, B::Float32(builder(rows, vals))),
        Type::FLOAT8 => (TI::Float64, B::Float64(builder(rows, vals))),
        Type::NUMERIC => (TI::Decimal128, B::Decimal128(builder(rows, vals))),

        Type::DATE => (TI::Timestamp, B::Timestamp(builder(rows, vals))),
        Type::TIME => (TI::Timestamp, B::Timestamp(builder(rows, vals))),
        Type::TIMETZ => (TI::Timestamp, B::Timestamp(builder(rows, vals))),
        Type::TIMESTAMP => (TI::Timestamp, B::Timestamp(builder(rows, vals))),
        Type::TIMESTAMPTZ => (TI::Timestamp, B::Timestamp(builder(rows, vals))),
        Type::INTERVAL => (TI::Duration, B::Duration(builder(rows, vals))),

        Type::CHAR | Type::VARCHAR | Type::TEXT | Type::NAME | Type::BPCHAR => {
            (TI::Utf8, B::Utf8(builder(rows, vals)))
        }

        Type::BYTEA => (TI::Binary, B::Binary(builder(rows, vals))),
        Type::UUID => (TI::Utf8, B::Utf8(builder(rows, vals))),
        Type::JSON | Type::JSONB => (TI::Utf8, B::Utf8(builder(rows, vals))),
        Type::XML => (TI::Utf8, B::Utf8(builder(rows, vals))),

        // Arrays - for now treat as text representation
        Type::BOOL_ARRAY
        | Type::INT2_ARRAY
        | Type::INT4_ARRAY
        | Type::INT8_ARRAY
        | Type::FLOAT4_ARRAY
        | Type::FLOAT8_ARRAY
        | Type::TEXT_ARRAY
        | Type::VARCHAR_ARRAY => (TI::Utf8, B::Utf8(builder(rows, vals))),

        // Network types as text
        Type::INET | Type::CIDR | Type::MACADDR | Type::MACADDR8 => {
            (TI::Utf8, B::Utf8(builder(rows, vals)))
        }

        // Geometric types as text
        Type::POINT
        | Type::LINE
        | Type::LSEG
        | Type::BOX
        | Type::PATH
        | Type::POLYGON
        | Type::CIRCLE => (TI::Utf8, B::Utf8(builder(rows, vals))),

        // Bit strings as binary
        Type::BIT | Type::VARBIT => (TI::Binary, B::Binary(builder(rows, vals))),

        // Money as text (for precision)
        Type::MONEY => (TI::Utf8, B::Utf8(builder(rows, vals))),

        // Range types as text
        Type::INT4_RANGE
        | Type::INT8_RANGE
        | Type::NUM_RANGE
        | Type::TS_RANGE
        | Type::TSTZ_RANGE
        | Type::DATE_RANGE => (TI::Utf8, B::Utf8(builder(rows, vals))),

        // Default: treat unknown types as text
        _ => (TI::Utf8, B::Utf8(builder(rows, vals))),
    }
}

fn builder<T: ValueStorage + FinishableStorage>(rows: usize, value_hint: usize) -> Builder<T> {
    Builder::with_capacity(true, rows, value_hint)
}

#[cfg(all(test, feature = "sqlx-postgres", feature = "arrow"))]
mod tests {
    use std::sync::Arc;

    use arrow_lib::datatypes::{DataType, Field as ArrowField, Schema, TimeUnit};
    use futures::StreamExt;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::prelude::*;

    use crate::encoding::{
        BatchSink, ColumnBinding, ColumnSpec, GenericRowEncoder, StrictCoercePolicy, TypeId,
    };
    use crate::formats::arrow::ArrowRecordBatchSink;

    fn type_id_to_arrow(type_id: &TypeId) -> DataType {
        match type_id {
            TypeId::Null => DataType::Null,
            TypeId::Boolean => DataType::Boolean,
            TypeId::Int8 => DataType::Int8,
            TypeId::Int16 => DataType::Int16,
            TypeId::Int32 => DataType::Int32,
            TypeId::Int64 => DataType::Int64,
            TypeId::UInt8 => DataType::UInt8,
            TypeId::UInt16 => DataType::UInt16,
            TypeId::UInt32 => DataType::UInt32,
            TypeId::UInt64 => DataType::UInt64,
            TypeId::Float32 => DataType::Float32,
            TypeId::Float64 => DataType::Float64,
            TypeId::Timestamp => DataType::Timestamp(TimeUnit::Millisecond, None),
            TypeId::Duration => DataType::Duration(TimeUnit::Nanosecond),
            TypeId::Decimal128 => DataType::Decimal128(38, 18),
            TypeId::Utf8 => DataType::Utf8,
            TypeId::LargeUtf8 => DataType::LargeUtf8,
            TypeId::Binary => DataType::Binary,
            TypeId::LargeBinary => DataType::LargeBinary,
            TypeId::FixedSizeBinary => DataType::FixedSizeBinary(16),
            TypeId::List(inner) => DataType::List(Arc::new(ArrowField::new(
                "item",
                type_id_to_arrow(inner),
                true,
            ))),
            TypeId::Struct(fields) => DataType::Struct(
                fields
                    .iter()
                    .enumerate()
                    .map(|(i, t)| ArrowField::new(format!("f{}", i), type_id_to_arrow(t), true))
                    .collect(),
            ),
        }
    }

    fn bindings_to_schema(bindings: &[ColumnBinding]) -> Schema {
        Schema::new(
            bindings
                .iter()
                .map(|b| {
                    ArrowField::new(
                        &b.field.name,
                        type_id_to_arrow(&b.field.type_id),
                        b.field.nullable,
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    #[tokio::test]
    async fn test_sqlx_postgres_to_arrow() {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:postgres@localhost:5432/coluna")
            .await
            .unwrap();

        // Prepare statement and get column info
        let stmt = pool
            .prepare("SELECT * FROM demo_types LIMIT 10")
            .await
            .unwrap();
        let columns = stmt.columns();
        println!("Columns: {}", columns.len());
        let bindings = columns.build_columns(100, 100);

        // Build Arrow schema from bindings
        let schema = Arc::new(bindings_to_schema(&bindings));
        println!("Schema: {:?}", schema);

        // Create encoder
        let mut encoder = GenericRowEncoder::new(bindings);

        // Fetch and encode rows
        let mut stream = stmt.query().fetch(&pool);
        let mut row_count = 0;
        while let Some(result) = stream.next().await {
            let row = result.unwrap();
            let policy = StrictCoercePolicy;
            encoder.append_row(&policy, row).unwrap();
            row_count += 1;
        }

        println!("Encoded {} rows", row_count);

        // Finish encoding and convert to Arrow arrays
        let sink = ArrowRecordBatchSink::new(schema.clone(), 1000);
        let column_data: Vec<_> = encoder.finish().collect();
        let batch = sink.sink(&[], column_data).unwrap();

        println!("RecordBatch: {:?}", batch);
        println!("Num rows: {}", batch.num_rows());
        println!("Num columns: {}", batch.num_columns());

        for (i, col) in batch.columns().iter().enumerate() {
            println!("Column {}: {} values", schema.field(i).name(), col.len());
        }

        assert_eq!(batch.num_rows(), row_count);
    }
}
