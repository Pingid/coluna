use crate::encoding::{CoercePolicy, ColumnBinding, ColumnData, RowAccess, RowError, SourceValue};

#[derive(Debug)]
pub struct GenericRowEncoder {
    pub columns: Vec<ColumnBinding>,
}

impl GenericRowEncoder {
    pub fn new(columns: Vec<ColumnBinding>) -> Self {
        Self { columns }
    }

    pub fn append_row<E, R: RowAccess<Error = E>, P: CoercePolicy>(
        &mut self, policy: &P, mut row: R,
    ) -> Result<(), RowError<E>> {
        for (i, col) in self.columns.iter_mut().enumerate() {
            let raw = row
                .get_value(col, i)
                .map_err(|e| RowError::source(col.field.name.clone(), e))?;

            col.builder
                .append_raw(policy, raw.unwrap_or(SourceValue::Null))
                .map_err(|e| RowError::coerce(col.field.name.clone(), e))?;
        }
        Ok(())
    }

    pub fn finish(&mut self) -> impl Iterator<Item = ColumnData> {
        self.columns.iter_mut().map(|c| c.builder.finish())
    }
}
