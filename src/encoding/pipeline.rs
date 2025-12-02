use super::{ColumnData, RowEncoder};
use crate::encoding::ColumnBinding;

pub trait BatchSink {
    type Error;
    type Batch;

    fn size(&self) -> usize;
    fn sink(
        &self, cols: &[ColumnBinding], data: Vec<ColumnData>,
    ) -> Result<Self::Batch, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum RowPipelineError<R: RowEncoder, B: BatchSink> {
    #[error("row {row}: {source}")]
    Row {
        row: usize,
        #[source]
        source: R::Error,
    },
    #[error("batch sink error: {0}")]
    BatchSink(#[source] B::Error),
}

pub struct RowPipeline<R, B> {
    row_encoder: R,
    batch_sink: B,
    rows_in_batch: usize,
    next_row_index: usize,
}

impl<R, B> RowPipeline<R, B>
where
    R: RowEncoder,
    B: BatchSink,
{
    pub fn new(row_encoder: R, batch_encoder: B) -> Self {
        Self {
            row_encoder,
            batch_sink: batch_encoder,
            rows_in_batch: 0,
            next_row_index: 0,
        }
    }

    pub fn append_row(&mut self, row: R::Row) -> Result<Option<B::Batch>, RowPipelineError<R, B>> {
        if let Err(source) = self.row_encoder.append(row) {
            return Err(RowPipelineError::Row {
                row: self.next_row_index,
                source,
            });
        }

        self.next_row_index += 1;
        self.rows_in_batch += 1;

        if self.rows_in_batch >= self.batch_sink.size() {
            let cols = self.row_encoder.finish();
            let batch = self
                .batch_sink
                .sink(self.row_encoder.columns(), cols)
                .map_err(RowPipelineError::BatchSink)?;
            self.rows_in_batch = 0;
            Ok(Some(batch))
        } else {
            Ok(None)
        }
    }

    pub fn finish(mut self) -> Result<Option<B::Batch>, RowPipelineError<R, B>> {
        if self.rows_in_batch == 0 {
            return Ok(None);
        }

        let cols = self.row_encoder.finish();
        self.batch_sink
            .sink(self.row_encoder.columns(), cols)
            .map_err(RowPipelineError::BatchSink)
            .map(Some)
    }
}
