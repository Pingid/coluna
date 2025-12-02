use crate::encoding::CoerceError;

#[derive(Debug, thiserror::Error)]
pub enum RowError<SourceErr> {
    #[error("source error in column `{column}`: {source}")]
    Source {
        column: String,
        #[source]
        source: SourceErr,
    },
    #[error("coerce error in column `{column}`: {source}")]
    Coerce {
        column: String,
        #[source]
        source: CoerceError,
    },
}

impl<SourceErr> RowError<SourceErr> {
    pub fn source(column: impl Into<String>, source: SourceErr) -> Self {
        Self::Source {
            column: column.into(),
            source,
        }
    }

    pub fn coerce(column: impl Into<String>, source: CoerceError) -> Self {
        Self::Coerce {
            column: column.into(),
            source,
        }
    }
}
