## Coluna

Encode row‑oriented data into column‑oriented structures.

Coluna provides building blocks for turning heterogeneous row data (JSON, database rows, etc.) into typed columns and Apache Arrow `RecordBatch`es.

### Features

- **Arrow targets**
  - Build `RecordBatch`es from encoded columns.
  - Support for nested types (struct, list) and temporal / decimal types where available.
- **Row sources**
  - `serde_json::Value` via `JsonRowEncoder`.
  - Postgres rows (module stubbed, API subject to change).
- **Coercion policies**
  - Numeric, temporal, and text coercion profiles (`StrictCoercePolicy`, `JsonCoercePolicy`).
- **Composable pipeline**
  - `RowEncoder` + `BatchSink` combined by `RowPipeline`.

### Installation

Add `coluna` to your `Cargo.toml`:

```toml
[dependencies]
coluna = { git = "https://github.com/Pingid/coluna" }
```

To target a specific Arrow version, select exactly one Arrow feature:

```toml
[dependencies]
coluna = { git = "https://github.com/Pingid/coluna", features = ["arrow-55", "json"] }
```

Available Arrow features: `arrow-54`, `arrow-55`, `arrow-56`, `arrow-57`.

### Example: JSON rows to Arrow `RecordBatch`

This example encodes JSON rows into Arrow `RecordBatch`es using the JSON source and Arrow sink.

```rust
use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema};
use serde_json::json;

use coluna::{ArrowRecordBatchSink, JsonRowEncoder, RowPipeline};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Input rows (serde_json::Value)
    let rows = vec![
        json!({ "name": "John", "age": 30 }),
        json!({ "name": "Jane", "age": 25 }),
        json!({ "name": "Jim", "age": 35 }),
    ];

    // Target Arrow schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, true),
        Field::new("age", DataType::Int64, true),
    ]));

    // Build encoder and sink
    let encoder = JsonRowEncoder::from_columns(&schema);
    let sink = ArrowRecordBatchSink::new(schema.clone(), 2);
    let mut pipeline = RowPipeline::new(encoder, sink);

    // Feed rows into the pipeline
    let mut batches = Vec::new();
    for row in rows {
        if let Some(batch) = pipeline.append_row(row)? {
            batches.push(batch);
        }
    }

    // Flush any remaining rows
    if let Some(batch) = pipeline.finish()? {
        batches.push(batch);
    }

    println!("produced {} batches", batches.len());
    Ok(())
}
```

### Goals

- Provide flexible abstractions for translating data between row and column structures.
- Support multiple row sources (JSON, PostgreSQL, other SQL databases) and multiple columnar targets.
- Keep the core encoding pipeline small, composable, and reusable across backends.
