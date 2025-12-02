use std::fmt;

use crate::encoding::SourceValue;

#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(usize),
}

#[derive(Debug, Clone, Default)]
pub struct ValuePath(Vec<PathSegment>);

impl ValuePath {
    pub fn root() -> Self {
        Self(Vec::new())
    }

    pub fn push_front(&mut self, seg: PathSegment) {
        self.0.insert(0, seg);
    }
}

impl fmt::Display for ValuePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        write!(f, " at ")?;
        let mut first = true;
        for seg in &self.0 {
            match seg {
                PathSegment::Field(name) => {
                    if first {
                        write!(f, "{name}")?;
                    } else {
                        write!(f, ".{name}")?;
                    }
                }
                PathSegment::Index(i) => {
                    write!(f, "[{i}]")?;
                }
            }
            first = false;
        }
        Ok(())
    }
}

/// Errors during casting / coercion from `RawValue` to target types.
#[derive(Debug, thiserror::Error)]
pub enum CoerceError {
    #[error("type mismatch{path}: expected {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
        path: ValuePath,
    },
}

impl CoerceError {
    pub fn mismatch(expected: impl Into<String>, found: &SourceValue) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            found: format!("{:?}", found),
            path: ValuePath::root(),
        }
    }

    pub fn at(self, seg: PathSegment) -> Self {
        match self {
            CoerceError::TypeMismatch {
                expected,
                found,
                mut path,
            } => {
                path.push_front(seg);
                CoerceError::TypeMismatch {
                    expected,
                    found,
                    path,
                }
            }
        }
    }
}
