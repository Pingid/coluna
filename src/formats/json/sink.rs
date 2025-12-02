use serde_json::Value as Json;

use crate::encoding::{BatchSink, ColumnBinding, ColumnData, ColumnValues};

#[derive(Debug)]
pub struct JsonBatchSink {
    batch_size: usize,
}

impl JsonBatchSink {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }
}

impl BatchSink for JsonBatchSink {
    type Error = ();
    type Batch = Vec<Json>;

    fn size(&self) -> usize {
        self.batch_size
    }

    fn sink(
        &self, cols: &[ColumnBinding], data: Vec<ColumnData>,
    ) -> Result<Self::Batch, Self::Error> {
        let row_count = data.first().map(|c| c.len()).unwrap_or(0);
        let mut iters: Vec<_> = data.into_iter().map(column_into_iter).collect();

        let mut results = Vec::with_capacity(row_count);
        for _ in 0..row_count {
            let mut row = serde_json::Map::with_capacity(cols.len());
            for (col, iter) in cols.iter().zip(iters.iter_mut()) {
                row.insert(col.field.name.clone(), iter.next().unwrap_or(Json::Null));
            }
            results.push(Json::Object(row));
        }
        Ok(results)
    }
}

/// Creates an iterator over Json values for a column. Type dispatch happens once here.
fn column_into_iter(col: ColumnData) -> Box<dyn Iterator<Item = Json>> {
    use ColumnValues as CV;
    let ColumnData { kind, validity } = col;

    macro_rules! primitive {
        ($v:expr, $conv:expr) => {
            nullable_iter(validity, $v.into_iter().map($conv))
        };
    }

    match kind {
        CV::Null => Box::new(validity.into_iter().map(|_| Json::Null)),

        CV::Boolean(v) => primitive!(v, Json::Bool),

        CV::Int8(v) => primitive!(v, |n| Json::Number((n as i64).into())),
        CV::Int16(v) => primitive!(v, |n| Json::Number((n as i64).into())),
        CV::Int32(v) => primitive!(v, |n| Json::Number((n as i64).into())),
        CV::Int64(v) => primitive!(v, |n| Json::Number(n.into())),

        CV::UInt8(v) => primitive!(v, |n| Json::Number((n as u64).into())),
        CV::UInt16(v) => primitive!(v, |n| Json::Number((n as u64).into())),
        CV::UInt32(v) => primitive!(v, |n| Json::Number((n as u64).into())),
        CV::UInt64(v) => primitive!(v, |n| Json::Number(n.into())),

        CV::Float32(v) => primitive!(v, |n| f64_to_json(n as f64)),
        CV::Float64(v) => primitive!(v, f64_to_json),

        CV::Timestamp(v) => primitive!(v, |t| Json::Number(t.0.into())),
        CV::Duration(v) => primitive!(v, |d| Json::Number(d.0.into())),

        CV::Decimal128(v) => primitive!(v, |n: i128| Json::String(n.to_string())),

        CV::Utf8 { offsets, values } => nullable_iter(validity, Utf8Iter::new(offsets, values)),
        CV::LargeUtf8 { offsets, values } => {
            nullable_iter(validity, LargeUtf8Iter::new(offsets, values))
        }

        CV::Binary { offsets, values } => nullable_iter(
            validity,
            BinaryIter::new(offsets, values).map(bytes_to_json),
        ),
        CV::LargeBinary { offsets, values } => nullable_iter(
            validity,
            LargeBinaryIter::new(offsets, values).map(bytes_to_json),
        ),

        CV::FixedSizeBinary { size, values } => nullable_iter(
            validity,
            FixedBinaryIter::new(size, values).map(bytes_to_json),
        ),

        CV::List {
            offsets,
            values,
            field,
        } => nullable_iter(validity, ListIter::new(offsets, *values, field.name)),

        CV::Struct { fields } => {
            let row_count = fields.first().map(|(_, data)| data.len()).unwrap_or(0);
            let names: Vec<_> = fields.iter().map(|(f, _)| f.name.clone()).collect();
            let mut iters: Vec<_> = fields
                .into_iter()
                .map(|(_, data)| column_into_iter(data))
                .collect();

            Box::new((0..row_count).zip(validity).map(move |(_, valid)| {
                if !valid {
                    iters.iter_mut().for_each(|it| {
                        it.next();
                    });
                    return Json::Null;
                }
                let mut obj = serde_json::Map::with_capacity(names.len());
                for (name, iter) in names.iter().zip(iters.iter_mut()) {
                    obj.insert(name.clone(), iter.next().unwrap_or(Json::Null));
                }
                Json::Object(obj)
            }))
        }
    }
}

/// Wraps a value iterator with validity, yielding Null for invalid positions.
fn nullable_iter<I: Iterator<Item = Json> + 'static>(
    validity: Vec<bool>, values: I,
) -> Box<dyn Iterator<Item = Json>> {
    if validity.is_empty() {
        Box::new(values)
    } else {
        Box::new(validity.into_iter().zip(values).map(
            |(valid, v)| {
                if valid { v } else { Json::Null }
            },
        ))
    }
}

fn f64_to_json(v: f64) -> Json {
    serde_json::Number::from_f64(v)
        .map(Json::Number)
        .unwrap_or(Json::Null)
}

fn bytes_to_json(bytes: &[u8]) -> Json {
    Json::Array(
        bytes
            .iter()
            .map(|&b| Json::Number((b as u64).into()))
            .collect(),
    )
}

// ─── Offset-based iterators ─────────────────────────────────────────────────

struct Utf8Iter {
    offsets: Vec<i32>,
    values: Vec<u8>,
    idx: usize,
}

impl Utf8Iter {
    fn new(offsets: Vec<i32>, values: Vec<u8>) -> Self {
        Self {
            offsets,
            values,
            idx: 0,
        }
    }
}

impl Iterator for Utf8Iter {
    type Item = Json;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx + 1 >= self.offsets.len() {
            return None;
        }
        let start = self.offsets[self.idx] as usize;
        let end = self.offsets[self.idx + 1] as usize;
        self.idx += 1;
        let s = String::from_utf8_lossy(&self.values[start..end]).into_owned();
        Some(Json::String(s))
    }
}

struct LargeUtf8Iter {
    offsets: Vec<i64>,
    values: Vec<u8>,
    idx: usize,
}

impl LargeUtf8Iter {
    fn new(offsets: Vec<i64>, values: Vec<u8>) -> Self {
        Self {
            offsets,
            values,
            idx: 0,
        }
    }
}

impl Iterator for LargeUtf8Iter {
    type Item = Json;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx + 1 >= self.offsets.len() {
            return None;
        }
        let start = self.offsets[self.idx] as usize;
        let end = self.offsets[self.idx + 1] as usize;
        self.idx += 1;
        let s = String::from_utf8_lossy(&self.values[start..end]).into_owned();
        Some(Json::String(s))
    }
}

struct BinaryIter {
    offsets: Vec<i32>,
    values: Vec<u8>,
    idx: usize,
}

impl BinaryIter {
    fn new(offsets: Vec<i32>, values: Vec<u8>) -> Self {
        Self {
            offsets,
            values,
            idx: 0,
        }
    }
}

impl Iterator for BinaryIter {
    type Item = &'static [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx + 1 >= self.offsets.len() {
            return None;
        }
        let start = self.offsets[self.idx] as usize;
        let end = self.offsets[self.idx + 1] as usize;
        self.idx += 1;
        // Safety: we own values and return a reference that lives as long as Self
        // We use unsafe to avoid cloning the slice data
        let slice = &self.values[start..end];
        Some(unsafe { std::mem::transmute::<&[u8], &'static [u8]>(slice) })
    }
}

struct LargeBinaryIter {
    offsets: Vec<i64>,
    values: Vec<u8>,
    idx: usize,
}

impl LargeBinaryIter {
    fn new(offsets: Vec<i64>, values: Vec<u8>) -> Self {
        Self {
            offsets,
            values,
            idx: 0,
        }
    }
}

impl Iterator for LargeBinaryIter {
    type Item = &'static [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx + 1 >= self.offsets.len() {
            return None;
        }
        let start = self.offsets[self.idx] as usize;
        let end = self.offsets[self.idx + 1] as usize;
        self.idx += 1;
        let slice = &self.values[start..end];
        Some(unsafe { std::mem::transmute::<&[u8], &'static [u8]>(slice) })
    }
}

struct FixedBinaryIter {
    size: usize,
    values: Vec<u8>,
    idx: usize,
}

impl FixedBinaryIter {
    fn new(size: i32, values: Vec<u8>) -> Self {
        Self {
            size: size as usize,
            values,
            idx: 0,
        }
    }
}

impl Iterator for FixedBinaryIter {
    type Item = &'static [u8];
    fn next(&mut self) -> Option<Self::Item> {
        let start = self.idx * self.size;
        let end = start + self.size;
        if end > self.values.len() {
            return None;
        }
        self.idx += 1;
        let slice = &self.values[start..end];
        Some(unsafe { std::mem::transmute::<&[u8], &'static [u8]>(slice) })
    }
}

struct ListIter {
    offsets: Vec<i32>,
    values: Box<dyn Iterator<Item = Json>>,
    _field_name: String,
    idx: usize,
}

impl ListIter {
    fn new(offsets: Vec<i32>, values: ColumnData, field_name: String) -> Self {
        Self {
            offsets,
            values: column_into_iter(values),
            _field_name: field_name,
            idx: 0,
        }
    }
}

impl Iterator for ListIter {
    type Item = Json;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx + 1 >= self.offsets.len() {
            return None;
        }
        let start = self.offsets[self.idx] as usize;
        let end = self.offsets[self.idx + 1] as usize;
        self.idx += 1;
        let arr: Vec<_> = (start..end).filter_map(|_| self.values.next()).collect();
        Some(Json::Array(arr))
    }
}
