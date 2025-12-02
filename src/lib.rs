#[cfg(feature = "arrow-54")]
extern crate arrow_v54 as arrow_lib;
#[cfg(feature = "arrow-55")]
extern crate arrow_v55 as arrow_lib;
#[cfg(feature = "arrow-56")]
extern crate arrow_v56 as arrow_lib;
#[cfg(feature = "arrow-57")]
extern crate arrow_v57 as arrow_lib;

mod encoding;
pub use encoding::RowPipeline;

mod formats;
pub use formats::*;

#[cfg(any(
    feature = "arrow",
    feature = "arrow-57",
    feature = "arrow-56",
    feature = "arrow-55",
    feature = "arrow-54"
))]
pub mod arrow {
    pub use arrow_lib::*;
}

#[cfg(all(test, feature = "json", feature = "arrow"))]
mod tests {
    use std::sync::Arc;

    use arrow_lib::datatypes::{DataType, Field, Schema};
    use serde_json::json;

    use super::*;

    #[test]
    fn json_encode_arrow_record_batch() {
        let rows = vec![
            // json!({ "name": "John", "age": 30 }),
            // json!({ "name": "Jane", "age": 25 }),
            json!({ "name": "Jim", "age": 35, "address": { "street": "123 Main St", "city": "Springfield", "state": "IL", "zip": "62704" } }),
        ];

        let schema = Arc::new(Schema::new(vec![
            Field::new("name", DataType::Utf8, true),
            Field::new("age", DataType::Int64, true),
            Field::new(
                "address",
                DataType::Struct(
                    vec![
                        Field::new("street", DataType::Utf8, true),
                        Field::new("city", DataType::Utf8, true),
                        Field::new("state", DataType::Utf8, true),
                        Field::new("zip", DataType::Utf8, true),
                    ]
                    .into(),
                ),
                true,
            ),
        ]));

        let encoder = JsonRowEncoder::from_spec(&schema);
        // let sink = ArrowRecordBatchSink::new(schema, 2);
        let sink = JsonBatchSink::new(usize::MAX);
        let mut pipeline = RowPipeline::new(encoder, sink);
        let mut batches = Vec::new();
        for row in rows.clone().into_iter() {
            if let Some(batch) = pipeline.append_row(row).unwrap() {
                batches.push(batch);
            }
        }
        if let Some(batch) = pipeline.finish().unwrap() {
            batches.push(batch);
        }
        assert_eq!(batches.into_iter().flatten().collect::<Vec<_>>(), rows);
    }
}
