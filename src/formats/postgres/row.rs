use crate::encoding::SourceValue;

#[derive(Debug, thiserror::Error)]
pub enum PgRowError {
    #[error("invalid row length: expected {0}, found {1}")]
    InvalidLength(usize, usize),
}

struct PgCol<'a> {
    value: &'a [u8],
    tp_name: &'a str,
}

impl<'a> PgCol<'a> {
    fn get_value(&mut self) -> Result<Option<SourceValue>, PgRowError> {
        match self.tp_name {
            "bool" => todo!(),
            _ => todo!(),
        }
    }
}
