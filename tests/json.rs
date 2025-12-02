#[cfg(any(feature = "arrow", feature = "arrow-57"))]
use coluna::arrow::datatypes;
#[cfg(feature = "json")]
use serde_json::json;

#[test]
#[cfg(feature = "json")]
#[cfg(any(feature = "arrow", feature = "arrow-57"))]
fn test_json_numeric_record_batch() {
    use datatypes::DataType::*;
    let rows = vec![
        json!({ "Int8": 1, "Int16": 2, "Int32": 3, "Int64": 4, "UInt8": 5, "UInt16": 6, "UInt32": 7, "UInt64": 8, "Float16": 9, "Float32": 10, "Float64": 11 }),
    ];

    let schema = utils::schema([
        ("Int8", Int8, false),
        ("Int16", Int16, false),
        ("Int32", Int32, false),
        ("Int64", Int64, false),
        ("UInt8", UInt8, false),
        ("UInt16", UInt16, false),
        ("UInt32", UInt32, false),
        ("UInt64", UInt64, false),
        ("Float16", Float16, false),
        ("Float32", Float32, false),
        ("Float64", Float64, false),
    ]);

    let batch = utils::encode_one_batch(&schema, rows);
    assert_eq!(
        utils::col_values::<datatypes::Int8Type>("Int8", &batch),
        vec![1],
    );
}

#[cfg(all(feature = "json", feature = "arrow-57"))]
mod utils {
    use std::sync::Arc;

    use coluna::arrow::datatypes::{DataType, Field, Fields, Schema};
    use coluna::{ArrowRecordBatchSink, JsonRowEncoder, RowPipeline, arrow};

    pub fn encode_batches(
        schema: &Arc<Schema>, batch_size: usize, rows: impl IntoIterator<Item = serde_json::Value>,
    ) -> Vec<arrow::array::RecordBatch> {
        let rows = rows.into_iter().collect::<Vec<_>>();

        let encoder = JsonRowEncoder::from_spec(schema);
        let sink = ArrowRecordBatchSink::new(schema.clone(), batch_size);
        let mut pipeline = RowPipeline::new(encoder, sink);
        let mut batches = Vec::new();

        for row in rows.clone() {
            if let Some(batch) = pipeline.append_row(row).expect("append_row failed") {
                batches.push(batch);
            }
        }

        let mut total_rows = 0;

        for batch in batches.iter() {
            assert_eq!(batch.num_rows(), batch_size);
            assert_eq!(batch.columns().len(), schema.fields().len());
            total_rows += batch.num_rows();
        }

        if let Some(batch) = pipeline.finish().expect("finish failed") {
            total_rows += batch.num_rows();
            batches.push(batch);
        }

        assert_eq!(total_rows, rows.into_iter().count());

        batches
    }

    pub fn encode_one_batch(
        schema: &Arc<Schema>, rows: impl IntoIterator<Item = serde_json::Value>,
    ) -> arrow::array::RecordBatch {
        let rows = rows.into_iter().collect::<Vec<_>>();
        let mut batches = encode_batches(&schema, usize::MAX, rows.clone());
        assert_eq!(batches.len(), 1, "expected a single batch");
        assert_eq!(batches[0].num_rows(), rows.len());
        assert_eq!(batches[0].columns().len(), schema.fields().len());
        batches.remove(0)
    }

    // ---------------- Arrow schema builder ----------------
    pub struct FieldType(Field);
    impl From<(&str, DataType)> for FieldType {
        fn from((name, dt): (&str, DataType)) -> Self {
            Self(Field::new(name, dt, false))
        }
    }
    impl From<(&str, DataType, bool)> for FieldType {
        fn from((name, dt, nullable): (&str, DataType, bool)) -> Self {
            Self(Field::new(name, dt, nullable))
        }
    }

    pub fn fields<T>(items: impl IntoIterator<Item = T>) -> Fields
    where T: Into<FieldType> {
        Fields::from(
            items
                .into_iter()
                .map(|item| item.into().0)
                .collect::<Vec<_>>(),
        )
    }

    pub fn schema<T>(items: impl IntoIterator<Item = T>) -> Arc<Schema>
    where T: Into<FieldType> {
        Arc::new(Schema::new(fields(items)))
    }

    // ---------------- Arrow record batch helpers ----------------
    pub fn col_values<T>(
        name: &str, batch: &arrow::array::RecordBatch,
    ) -> Vec<<T as arrow::datatypes::ArrowPrimitiveType>::Native>
    where T: arrow::datatypes::ArrowPrimitiveType {
        let arr = col::<arrow::array::PrimitiveArray<T>>(name, batch);
        let mut values = Vec::new();
        for i in 0..arr.len() {
            values.push(arr.value(i));
        }
        values
    }

    pub fn col<'a, A>(name: &str, batch: &'a arrow::array::RecordBatch) -> &'a A
    where A: arrow::array::Array + 'static {
        let idx = batch.schema().index_of(name).unwrap();
        batch
            .column(idx)
            .as_any()
            .downcast_ref::<A>()
            .unwrap_or_else(|| panic!("column `{name}` has unexpected type"))
    }
}
