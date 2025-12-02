use serde_json::Value as Json;

use super::access::{DefaultJsonFieldAccess, JsonFieldAccess};
use crate::encoding::{
    CoercePolicy, ColumnBinding, ColumnData, ColumnSpec, GenericRowEncoder, RowAccess, RowEncoder,
    RowError, SourceValue, StrictCoercePolicy,
};

#[derive(Debug, thiserror::Error)]
pub enum JsonRowError {
    #[error("type mismatch: expected {0}, found {1}")]
    TypeMismatch(String, String),
    #[error("decode error: {0}")]
    Decode(String),
}

impl JsonRowError {
    pub fn mismatch(expected: impl Into<String>, found: &Json) -> Self {
        Self::TypeMismatch(expected.into(), found.to_string())
    }
}

#[derive(Debug)]
pub struct JsonRowEncoder<
    P: CoercePolicy = StrictCoercePolicy,
    A: JsonFieldAccess = DefaultJsonFieldAccess,
> {
    inner: GenericRowEncoder,
    access: A,
    policy: P,
}

impl JsonRowEncoder<StrictCoercePolicy, DefaultJsonFieldAccess> {
    pub fn from_spec<T: ColumnSpec>(columns: &T) -> Self {
        let columns = columns.build_columns(0, 0);
        Self {
            inner: GenericRowEncoder::new(columns),
            access: DefaultJsonFieldAccess,
            policy: StrictCoercePolicy,
        }
    }
}

impl<P: CoercePolicy, A: JsonFieldAccess> RowEncoder for JsonRowEncoder<P, A>
where A::Error: Into<JsonRowError>
{
    type Error = RowError<JsonRowError>;
    type Row = Json;

    fn columns(&self) -> &[ColumnBinding] {
        &self.inner.columns
    }

    fn append(&mut self, row: Self::Row) -> Result<(), Self::Error> {
        self.inner.append_row(
            &self.policy,
            JsonRow {
                value: row,
                access: &self.access,
            },
        )
    }

    fn finish(&mut self) -> Vec<ColumnData> {
        self.inner.finish().collect()
    }
}

struct JsonRow<'a, A> {
    value: Json,
    access: &'a A,
}

impl<'a, A: JsonFieldAccess> RowAccess for JsonRow<'a, A>
where A::Error: Into<JsonRowError>
{
    type Error = JsonRowError;

    fn get_value(
        &mut self, col: &ColumnBinding, _idx: usize,
    ) -> Result<Option<SourceValue>, Self::Error> {
        match &mut self.value {
            Json::Object(obj) => {
                let c = obj.remove(col.field.name.as_str()).unwrap_or(Json::Null);
                self.access.get_value(&col, c).map_err(Into::into).map(Some)
            }
            _ => return Err(JsonRowError::mismatch("object", &self.value)),
        }
    }
}
