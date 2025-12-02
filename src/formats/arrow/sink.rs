use std::sync::Arc;

use arrow_lib::array::RecordBatch;
use arrow_lib::datatypes::Schema;
use arrow_lib::error::ArrowError;

use super::array::column_to_array;
use crate::encoding::{BatchSink, ColumnBinding, ColumnData};

#[derive(Debug)]
pub struct ArrowRecordBatchSink {
    schema: Arc<Schema>,
    batch_size: usize,
}

impl ArrowRecordBatchSink {
    pub fn new(schema: Arc<Schema>, batch_size: usize) -> Self {
        Self { schema, batch_size }
    }
}

impl BatchSink for ArrowRecordBatchSink {
    type Error = ArrowError;
    type Batch = RecordBatch;

    fn size(&self) -> usize {
        self.batch_size
    }

    fn sink(&self, _: &[ColumnBinding], data: Vec<ColumnData>) -> Result<Self::Batch, Self::Error> {
        let mut arrays = Vec::new();
        for col in data.into_iter().enumerate() {
            arrays.push(column_to_array(&self.schema.field(col.0), col.1)?);
        }
        RecordBatch::try_new(self.schema.clone(), arrays)
    }
}
