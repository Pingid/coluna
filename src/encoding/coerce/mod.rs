use super::{DurationNano, SourceValue, TemporalNano};

mod error;
pub use error::*;

/// Numeric casting profile: “any number to any number” lives here.
pub trait NumericPolicy {
    fn as_i64(&self, raw: SourceValue) -> Result<Option<i64>, CoerceError>;
    fn as_f64(&self, raw: SourceValue) -> Result<Option<f64>, CoerceError>;
}

/// Temporal casting profile: timestamps / dates.
pub trait TemporalPolicy {
    /// Timestamp in milliseconds since Unix epoch.
    fn coerce_timestamp(&self, raw: SourceValue) -> Result<Option<TemporalNano>, CoerceError>;
    /// Duration in nanoseconds.
    fn coerce_duration(&self, raw: SourceValue) -> Result<Option<DurationNano>, CoerceError>;
}

/// Text casting profile.
pub trait TextPolicy {
    fn as_str(&self, raw: SourceValue) -> Result<Option<String>, CoerceError>;
    fn as_binary(&self, raw: SourceValue) -> Result<Option<Vec<u8>>, CoerceError>;
}

/// Aggregate policy trait encoders depend on.
/// Backends pick a concrete implementation (e.g. `JsonCastPolicy`).
pub trait CoercePolicy: NumericPolicy + TemporalPolicy + TextPolicy {
    fn coerce_null(&self, _: SourceValue) -> Result<Option<()>, CoerceError> {
        Ok(Some(()))
    }

    // --- basic / boolean ---

    fn coerce_bool(&self, raw: SourceValue) -> Result<Option<bool>, CoerceError> {
        match raw {
            SourceValue::Null => Ok(None),
            SourceValue::Boolean(b) => Ok(Some(b)),
            other => Err(CoerceError::mismatch("bool", &other)),
        }
    }

    // --- integer family, via `as_i64` ---

    fn coerce_i8(&self, raw: SourceValue) -> Result<Option<i8>, CoerceError> {
        if let SourceValue::Int8(v) = raw {
            return Ok(Some(v));
        }
        let v = match self.as_i64(raw)? {
            None => return Ok(None),
            Some(v) => v,
        };
        if v < i8::MIN as i64 || v > i8::MAX as i64 {
            return Err(CoerceError::mismatch("i8 in range", &SourceValue::Int64(v)));
        }
        Ok(Some(v as i8))
    }

    fn coerce_i16(&self, raw: SourceValue) -> Result<Option<i16>, CoerceError> {
        if let SourceValue::Int16(v) = raw {
            return Ok(Some(v));
        }
        let v = match self.as_i64(raw)? {
            None => return Ok(None),
            Some(v) => v,
        };
        if v < i16::MIN as i64 || v > i16::MAX as i64 {
            return Err(CoerceError::mismatch(
                "i16 in range",
                &SourceValue::Int64(v),
            ));
        }
        Ok(Some(v as i16))
    }

    fn coerce_i32(&self, raw: SourceValue) -> Result<Option<i32>, CoerceError> {
        if let SourceValue::Int32(v) = raw {
            return Ok(Some(v));
        }
        let v = match self.as_i64(raw)? {
            None => return Ok(None),
            Some(v) => v,
        };
        if v < i32::MIN as i64 || v > i32::MAX as i64 {
            return Err(CoerceError::mismatch(
                "i32 in range",
                &SourceValue::Int64(v),
            ));
        }
        Ok(Some(v as i32))
    }

    fn coerce_i64(&self, raw: SourceValue) -> Result<Option<i64>, CoerceError> {
        self.as_i64(raw)
    }

    fn coerce_u8(&self, raw: SourceValue) -> Result<Option<u8>, CoerceError> {
        if let SourceValue::UInt8(v) = raw {
            return Ok(Some(v));
        }
        let v = match self.as_i64(raw)? {
            None => return Ok(None),
            Some(v) => v,
        };
        if v < u8::MIN as i64 || v > u8::MAX as i64 {
            return Err(CoerceError::mismatch("u8 in range", &SourceValue::Int64(v)));
        }
        Ok(Some(v as u8))
    }

    fn coerce_u16(&self, raw: SourceValue) -> Result<Option<u16>, CoerceError> {
        if let SourceValue::UInt16(v) = raw {
            return Ok(Some(v));
        }
        let v = match self.as_i64(raw)? {
            None => return Ok(None),
            Some(v) => v,
        };
        if v < u16::MIN as i64 || v > u16::MAX as i64 {
            return Err(CoerceError::mismatch(
                "u16 in range",
                &SourceValue::Int64(v),
            ));
        }
        Ok(Some(v as u16))
    }

    fn coerce_u32(&self, raw: SourceValue) -> Result<Option<u32>, CoerceError> {
        if let SourceValue::UInt32(v) = raw {
            return Ok(Some(v));
        }
        let v = match self.as_i64(raw)? {
            None => return Ok(None),
            Some(v) => v,
        };
        if v < u32::MIN as i64 || v > u32::MAX as i64 {
            return Err(CoerceError::mismatch(
                "u32 in range",
                &SourceValue::Int64(v),
            ));
        }
        Ok(Some(v as u32))
    }

    fn coerce_u64(&self, raw: SourceValue) -> Result<Option<u64>, CoerceError> {
        if let SourceValue::UInt64(v) = raw {
            return Ok(Some(v));
        }
        let v = match self.as_i64(raw)? {
            None => return Ok(None),
            Some(v) => v,
        };
        if v < 0 {
            return Err(CoerceError::mismatch(
                "non-negative",
                &SourceValue::Int64(v),
            ));
        }
        Ok(Some(v as u64))
    }

    // --- float family, via `as_f64` ---

    fn coerce_f32(&self, raw: SourceValue) -> Result<Option<f32>, CoerceError> {
        if let SourceValue::Float32(v) = raw {
            return Ok(Some(v));
        }
        Ok(self.as_f64(raw)?.map(|v| v as f32))
    }

    fn coerce_f64(&self, raw: SourceValue) -> Result<Option<f64>, CoerceError> {
        if let SourceValue::Float64(v) = raw {
            return Ok(Some(v));
        }
        self.as_f64(raw)
    }

    // --- decimal ---

    fn coerce_decimal128(&self, raw: SourceValue) -> Result<Option<i128>, CoerceError> {
        if let SourceValue::Decimal128(v) = raw {
            return Ok(Some(v));
        }
        use SourceValue::*;
        Ok(match raw {
            Null => None,
            Decimal128(v) => Some(v),
            Int8(v) => Some(v as i128),
            Int16(v) => Some(v as i128),
            Int32(v) => Some(v as i128),
            Int64(v) => Some(v as i128),
            UInt8(v) => Some(v as i128),
            UInt16(v) => Some(v as i128),
            UInt32(v) => Some(v as i128),
            UInt64(v) => Some(v as i128),
            other => return Err(CoerceError::mismatch("decimal128", &other)),
        })
    }

    // --- text / temporal ---

    fn coerce_str(&self, raw: SourceValue) -> Result<Option<String>, CoerceError> {
        self.as_str(raw)
    }

    fn coerce_binary(&self, raw: SourceValue) -> Result<Option<Vec<u8>>, CoerceError> {
        self.as_binary(raw)
    }
}

// Concrete policy building blocks
//

/// Strict numeric policy: only numeric `RawValue` (no string parsing).
#[derive(Debug, Default, Clone, Copy)]
pub struct Numeric;

impl NumericPolicy for Numeric {
    fn as_i64(&self, raw: SourceValue) -> Result<Option<i64>, CoerceError> {
        use SourceValue::*;
        Ok(match raw {
            Null => None,
            Int8(v) => Some(v as i64),
            Int16(v) => Some(v as i64),
            Int32(v) => Some(v as i64),
            Int64(v) => Some(v),
            UInt8(v) => Some(v as i64),
            UInt16(v) => Some(v as i64),
            UInt32(v) => Some(v as i64),
            UInt64(v) => {
                if v > i64::MAX as u64 {
                    return Err(CoerceError::mismatch("i64 range", &raw));
                }
                Some(v as i64)
            }
            Float32(v) => Some(v as i64),
            Float64(v) => Some(v as i64),
            other => return Err(CoerceError::mismatch("numeric", &other)),
        })
    }

    fn as_f64(&self, raw: SourceValue) -> Result<Option<f64>, CoerceError> {
        use SourceValue::*;
        Ok(match raw {
            Null => None,
            Int8(v) => Some(v as f64),
            Int16(v) => Some(v as f64),
            Int32(v) => Some(v as f64),
            Int64(v) => Some(v as f64),
            UInt8(v) => Some(v as f64),
            UInt16(v) => Some(v as f64),
            UInt32(v) => Some(v as f64),
            UInt64(v) => Some(v as f64),
            Float32(v) => Some(v as f64),
            Float64(v) => Some(v),
            other => return Err(CoerceError::mismatch("numeric", &other)),
        })
    }
}

/// Strict temporal policy: only accepts dedicated temporal variants (none yet).
#[derive(Debug, Default, Clone, Copy)]
pub struct StrictTemporal;

impl TemporalPolicy for StrictTemporal {
    fn coerce_timestamp(&self, raw: SourceValue) -> Result<Option<TemporalNano>, CoerceError> {
        match raw {
            SourceValue::Null => Ok(None),
            SourceValue::Timestamp(v) => Ok(Some(v)),
            other => Err(CoerceError::mismatch("timestamp", &other)),
        }
    }

    fn coerce_duration(&self, raw: SourceValue) -> Result<Option<DurationNano>, CoerceError> {
        match raw {
            SourceValue::Null => Ok(None),
            SourceValue::Duration(v) => Ok(Some(v)),
            other => Err(CoerceError::mismatch("duration", &other)),
        }
    }
}

/// Simple text policy:
/// - `Str` passes through
/// - everything else becomes `format!("{:?}", ...)`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SimpleText;

impl TextPolicy for SimpleText {
    fn as_str(&self, raw: SourceValue) -> Result<Option<String>, CoerceError> {
        Ok(match raw {
            SourceValue::Null => None,
            SourceValue::Utf8(s) => Some(s),
            SourceValue::Binary(b) => Some(String::from_utf8_lossy(&b).to_string()),
            other => Some(format!("{:?}", other)),
        })
    }

    fn as_binary(&self, raw: SourceValue) -> Result<Option<Vec<u8>>, CoerceError> {
        Ok(match raw {
            SourceValue::Null => None,
            SourceValue::Binary(b) => Some(b),
            SourceValue::Utf8(s) => Some(s.as_bytes().to_vec()),
            other => return Err(CoerceError::mismatch("binary", &other)),
        })
    }
}

/// Strict policy: strict numbers + strict temporal + simple text.
#[derive(Debug, Default, Clone, Copy)]
pub struct StrictCoercePolicy;

impl NumericPolicy for StrictCoercePolicy {
    fn as_i64(&self, raw: SourceValue) -> Result<Option<i64>, CoerceError> {
        Numeric.as_i64(raw)
    }

    fn as_f64(&self, raw: SourceValue) -> Result<Option<f64>, CoerceError> {
        Numeric.as_f64(raw)
    }
}

impl TemporalPolicy for StrictCoercePolicy {
    fn coerce_timestamp(&self, raw: SourceValue) -> Result<Option<TemporalNano>, CoerceError> {
        StrictTemporal.coerce_timestamp(raw)
    }

    fn coerce_duration(&self, raw: SourceValue) -> Result<Option<DurationNano>, CoerceError> {
        StrictTemporal.coerce_duration(raw)
    }
}

impl TextPolicy for StrictCoercePolicy {
    fn as_str(&self, raw: SourceValue) -> Result<Option<String>, CoerceError> {
        SimpleText.as_str(raw)
    }

    fn as_binary(&self, raw: SourceValue) -> Result<Option<Vec<u8>>, CoerceError> {
        SimpleText.as_binary(raw)
    }
}

impl CoercePolicy for StrictCoercePolicy {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_numeric() {
        assert_coerce(
            "int64 pass-through",
            |v| StrictCoercePolicy.coerce_i64(v),
            SourceValue::Int64(42),
            Some(42),
        );
    }

    pub fn assert_coerce<T>(
        name: &str, f: impl Fn(SourceValue) -> Result<Option<T>, CoerceError>, raw: SourceValue,
        expected: Option<T>,
    ) where
        T: PartialEq + std::fmt::Debug,
    {
        let got = f(raw.clone()).unwrap_or_else(|e| panic!("{name}: {e} (raw: {raw:?})"));
        assert_eq!(got, expected, "{name}: mismatch");
    }
}
