use crate::encoding::{ColumnData, FinishableStorage, ValueStorage};

#[derive(Debug, Clone)]
pub enum Builder<S> {
    Null(NullableBuilder<S>),
    NonNull(S),
}

impl<S: ValueStorage> Builder<S> {
    pub fn new_with_capacity(storage: S, nullable: bool, rows: usize) -> Self {
        match nullable {
            true => Self::Null(NullableBuilder::new_with_capacity(storage, rows)),
            false => Self::NonNull(storage),
        }
    }

    pub fn with_capacity(nullable: bool, rows: usize, value_hint: usize) -> Self {
        match nullable {
            true => Self::Null(NullableBuilder::new_with_capacity(
                S::with_capacity(rows, value_hint),
                rows,
            )),
            false => Self::NonNull(S::with_capacity(rows, value_hint)),
        }
    }
}

impl<S: ValueStorage + FinishableStorage> Builder<S> {
    pub fn len(&self) -> usize {
        match self {
            Builder::Null(s) => s.len(),
            Builder::NonNull(s) => s.len(),
        }
    }

    pub fn append_null(&mut self) {
        match self {
            Builder::Null(s) => s.append_null(),
            Builder::NonNull(_) => panic!("Cannot append null to non-nullable builder"),
        }
    }

    pub fn append(&mut self, v: Option<S::Item>) {
        match self {
            Builder::Null(s) => s.append(v),
            Builder::NonNull(s) => match v {
                Some(v) => s.push(v),
                None => panic!("Cannot append null to non-nullable builder"),
            },
        }
    }

    pub fn finish(&mut self) -> ColumnData {
        match self {
            Builder::Null(s) => s.finish(),
            Builder::NonNull(s) => ColumnData {
                kind: s.finish(),
                validity: Vec::new(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct NullableBuilder<S> {
    pub storage: S,
    pub validity: Vec<bool>,
}

impl<S> NullableBuilder<S> {
    pub fn new_with_capacity(storage: S, rows: usize) -> Self {
        Self {
            storage,
            validity: Vec::with_capacity(rows),
        }
    }
}

impl<S> NullableBuilder<S>
where S: ValueStorage + FinishableStorage
{
    #[inline]
    pub fn append(&mut self, v: Option<S::Item>) {
        match v {
            Some(x) => self.append_some(x),
            None => self.append_null(),
        }
    }

    #[inline]
    pub fn append_some(&mut self, v: S::Item) {
        self.storage.push(v);
        self.validity.push(true);
    }

    #[inline]
    pub fn append_null(&mut self) {
        self.validity.push(false);
        self.storage.push_null();
    }

    pub fn len(&self) -> usize {
        self.validity.len()
    }

    pub fn finish(&mut self) -> ColumnData {
        let len = self.validity.len();
        let validity = std::mem::replace(&mut self.validity, Vec::with_capacity(len));
        ColumnData {
            kind: self.storage.finish(),
            validity,
        }
    }
}
