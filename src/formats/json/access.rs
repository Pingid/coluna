use serde_json::Value as Json;

use super::JsonRowError;
use crate::encoding::{ColumnBinding, Numeric, NumericPolicy, SourceValue, TemporalNano, TypeId};

macro_rules! json_field_access_def {
    ($(($tp:pat_param, $f:ident)),* $(,)?) => {
        fn get_value(&self, col: &ColumnBinding, value: Json) -> Result<SourceValue, Self::Error> {
            match col.field.type_id {
                $( $tp => self.$f(&col.field.name, value), )*
            }
        }
        $( fn $f(&self, _name: &str, value: Json) -> Result<SourceValue, Self::Error> { Ok(value.into()) } )*
    };
}

pub trait JsonFieldAccess {
    type Error;
    crate::access_types!(json_field_access_def);
}

impl From<Json> for SourceValue {
    fn from(v: Json) -> Self {
        match v {
            Json::Null => SourceValue::Null,
            Json::Bool(b) => SourceValue::Boolean(b),
            Json::Number(n) => json_number_to_source_value(n),
            Json::String(s) => SourceValue::Utf8(s),
            Json::Array(arr) => SourceValue::List(arr.into_iter().map(SourceValue::from).collect()),
            Json::Object(obj) => SourceValue::Struct(
                obj.into_iter()
                    .map(|(k, v)| (k, SourceValue::from(v)))
                    .collect(),
            ),
        }
    }
}

fn json_number_to_source_value(n: serde_json::Number) -> SourceValue {
    if n.is_f64() {
        SourceValue::Float64(n.as_f64().unwrap())
    } else if n.is_i64() {
        SourceValue::Int64(n.as_i64().unwrap())
    } else if n.is_u64() {
        SourceValue::UInt64(n.as_u64().unwrap())
    } else {
        SourceValue::Null
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultJsonFieldAccess;

impl JsonFieldAccess for DefaultJsonFieldAccess {
    type Error = JsonRowError;

    fn get_timestamp(&self, _name: &str, value: Json) -> Result<SourceValue, Self::Error> {
        match value {
            Json::String(s) => {
                let dt = chrono::DateTime::parse_from_rfc3339(&s)
                    .or_else(|_| chrono::DateTime::parse_from_rfc2822(&s))
                    .map_err(|_| JsonRowError::Decode(format!("invalid timestamp: {}", s)))?;
                Ok(SourceValue::Timestamp(TemporalNano::from_millis(
                    dt.timestamp_millis(),
                )))
            }
            Json::Number(n) => {
                let e = JsonRowError::Decode(format!("invalid timestamp: {:?}", n));
                let v = json_number_to_source_value(n);
                if let Ok(Some(v)) = Numeric.as_i64(v) {
                    return Ok(SourceValue::Timestamp(TemporalNano::from_infer(v)));
                }
                return Err(e);
            }
            other => {
                return Err(JsonRowError::Decode(format!(
                    "invalid timestamp: {}",
                    other
                )));
            }
        }
    }

    fn get_utf8(&self, _name: &str, value: Json) -> Result<SourceValue, Self::Error> {
        match value {
            Json::String(s) => Ok(SourceValue::Utf8(s)),
            other => Ok(SourceValue::Utf8(serde_json::to_string(&other).unwrap())),
        }
    }

    fn get_large_utf8(&self, name: &str, value: Json) -> Result<SourceValue, Self::Error> {
        self.get_utf8(name, value)
    }
}
