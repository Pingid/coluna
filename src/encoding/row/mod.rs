use crate::encoding::{ColumnBinding, ColumnData, SourceValue, TypeId};

mod encode;
mod error;

pub use encode::*;
pub use error::*;

pub trait RowEncoder {
    type Error;
    type Row;

    fn columns(&self) -> &[ColumnBinding];
    fn append(&mut self, row: Self::Row) -> Result<(), Self::Error>;
    fn finish(&mut self) -> Vec<ColumnData>;
}

macro_rules! row_access_def {
    ($(($tp:pat_param, $f:ident)),* $(,)?) => {
        fn get_value(&mut self, col: &ColumnBinding, idx: usize) -> Result<Option<SourceValue>, Self::Error> {
            match col.field.type_id {
                $( $tp => self.$f(col, idx), )*
            }
        }
        $( fn $f(&mut self, _col: &ColumnBinding, _idx: usize) -> Result<Option<SourceValue>, Self::Error> { Ok(None) } )*
    };
}

pub trait RowAccess {
    type Error;

    crate::access_types!(row_access_def);
}
