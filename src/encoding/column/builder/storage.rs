use std::marker::PhantomData;

use crate::encoding::{ColumnValues, DurationNano, TemporalNano};

pub trait ValueStorage {
    type Item;

    fn with_capacity(rows: usize, value_hint: usize) -> Self;
    fn len(&self) -> usize;
    fn push(&mut self, v: Self::Item);
    fn push_null(&mut self) {}
}

pub trait FinishableStorage {
    fn finish(&mut self) -> ColumnValues;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NullStorage;

impl ValueStorage for NullStorage {
    type Item = ();

    fn with_capacity(_rows: usize, _value_hint: usize) -> Self {
        NullStorage
    }

    fn len(&self) -> usize {
        0
    }

    fn push(&mut self, _v: ()) {}
}

impl FinishableStorage for NullStorage {
    fn finish(&mut self) -> ColumnValues {
        ColumnValues::Null
    }
}

#[derive(Debug, Clone)]
pub struct PrimitiveStorage<T>(pub Vec<T>);

impl<T: Default> ValueStorage for PrimitiveStorage<T> {
    type Item = T;

    fn with_capacity(rows: usize, _value_hint: usize) -> Self {
        Self(Vec::with_capacity(rows))
    }

    fn push(&mut self, v: Self::Item) {
        self.0.push(v);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn push_null(&mut self) {
        self.0.push(T::default());
    }
}

macro_rules! primitive_buffer_kind {
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl FinishableStorage for PrimitiveStorage<$type> {
                fn finish(&mut self) -> ColumnValues {
                    let len = self.0.len();
                    let items = std::mem::replace(&mut self.0, Vec::with_capacity(len));
                    ColumnValues::$variant(items)
                }
            }
        )*
    };
}

primitive_buffer_kind!(
    Boolean => bool,
    Int8 => i8,
    Int16 => i16,
    Int32 => i32,
    Int64 => i64,
    UInt8 => u8,
    UInt16 => u16,
    UInt32 => u32,
    UInt64 => u64,
    Float32 => f32,
    Float64 => f64,
    Timestamp => TemporalNano,
    Duration => DurationNano,
    Decimal128 => i128,
);

/// ---- Generic var-len storage (replaces SliceStorage enum) ----

pub trait Offset: Copy + Default {
    fn add_len(self, len: usize) -> Self;
}

impl Offset for i32 {
    #[inline]
    fn add_len(self, len: usize) -> Self {
        self + len as i32
    }
}

impl Offset for i64 {
    #[inline]
    fn add_len(self, len: usize) -> Self {
        self + len as i64
    }
}

pub trait VarLenKind {
    type Offset: Offset;

    fn into_column_values(offsets: Vec<Self::Offset>, values: Vec<u8>) -> ColumnValues;
}

#[derive(Debug, Clone)]
pub struct VarLenStorage<K: VarLenKind> {
    offsets: Vec<K::Offset>,
    values: Vec<u8>,
    _marker: PhantomData<K>,
}

impl<K: VarLenKind> VarLenStorage<K> {
    pub fn new() -> Self {
        Self {
            offsets: vec![K::Offset::default()],
            values: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(rows: usize, bytes_hint: usize) -> Self {
        let mut offsets = Vec::with_capacity(rows + 1);
        offsets.push(K::Offset::default());
        Self {
            offsets,
            values: Vec::with_capacity(bytes_hint),
            _marker: PhantomData,
        }
    }
}

impl<K: VarLenKind> Default for VarLenStorage<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: VarLenKind> ValueStorage for VarLenStorage<K> {
    type Item = Vec<u8>;

    fn with_capacity(rows: usize, bytes_hint: usize) -> Self {
        Self::with_capacity(rows, bytes_hint)
    }

    fn push(&mut self, s: Vec<u8>) {
        self.values.extend_from_slice(&s);
        let last = *self.offsets.last().unwrap();
        self.offsets.push(last.add_len(s.len()));
    }

    fn push_null(&mut self) {
        let last = *self.offsets.last().unwrap();
        self.offsets.push(last);
    }

    fn len(&self) -> usize {
        self.offsets.len() - 1
    }
}

impl<K: VarLenKind> FinishableStorage for VarLenStorage<K> {
    fn finish(&mut self) -> ColumnValues {
        let offsets = std::mem::take(&mut self.offsets);
        let values = std::mem::take(&mut self.values);
        K::into_column_values(offsets, values)
    }
}

/// ---- Concrete kinds + type aliases ----

#[derive(Debug, Clone)]
pub struct Utf8Kind;
#[derive(Debug, Clone)]
pub struct BinaryKind;
#[derive(Debug, Clone)]
pub struct LargeUtf8Kind;

#[derive(Debug, Clone)]
pub struct LargeBinaryKind;

impl VarLenKind for Utf8Kind {
    type Offset = i32;

    fn into_column_values(offsets: Vec<Self::Offset>, values: Vec<u8>) -> ColumnValues {
        ColumnValues::Utf8 { offsets, values }
    }
}

impl VarLenKind for BinaryKind {
    type Offset = i32;

    fn into_column_values(offsets: Vec<Self::Offset>, values: Vec<u8>) -> ColumnValues {
        ColumnValues::Binary { offsets, values }
    }
}

impl VarLenKind for LargeUtf8Kind {
    type Offset = i64;

    fn into_column_values(offsets: Vec<Self::Offset>, values: Vec<u8>) -> ColumnValues {
        ColumnValues::LargeUtf8 { offsets, values }
    }
}

impl VarLenKind for LargeBinaryKind {
    type Offset = i64;

    fn into_column_values(offsets: Vec<Self::Offset>, values: Vec<u8>) -> ColumnValues {
        ColumnValues::LargeBinary { offsets, values }
    }
}

pub type Utf8Storage = VarLenStorage<Utf8Kind>;
pub type BinaryStorage = VarLenStorage<BinaryKind>;
pub type LargeUtf8Storage = VarLenStorage<LargeUtf8Kind>;
pub type LargeBinaryStorage = VarLenStorage<LargeBinaryKind>;

#[derive(Debug, Clone)]
pub struct FixedSizeBinaryStorage {
    pub size: i32,
    pub values: Vec<u8>,
}

impl FixedSizeBinaryStorage {
    pub fn new(size: i32) -> Self {
        Self {
            size,
            values: Vec::new(),
        }
    }

    pub fn with_capacity(size: i32, rows: usize) -> Self {
        Self {
            size,
            values: Vec::with_capacity(rows * size as usize),
        }
    }
}

impl ValueStorage for FixedSizeBinaryStorage {
    type Item = Vec<u8>;

    fn with_capacity(_rows: usize, _value_hint: usize) -> Self {
        panic!(
            "FixedSizeBinaryStorage requires size parameter, use FixedSizeBinaryStorage::with_capacity(size, rows)"
        )
    }

    fn push(&mut self, v: Vec<u8>) {
        assert_eq!(
            v.len(),
            self.size as usize,
            "FixedSizeBinary value must be exactly {} bytes",
            self.size
        );
        self.values.extend_from_slice(&v);
    }

    fn push_null(&mut self) {
        self.values
            .extend(std::iter::repeat(0).take(self.size as usize));
    }

    fn len(&self) -> usize {
        self.values.len() / self.size as usize
    }
}

impl FinishableStorage for FixedSizeBinaryStorage {
    fn finish(&mut self) -> ColumnValues {
        let values = std::mem::take(&mut self.values);
        ColumnValues::FixedSizeBinary {
            size: self.size,
            values,
        }
    }
}
