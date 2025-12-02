use crate::encoding::{
    CoerceError, CoercePolicy, ColumnBinding, ColumnBuilder, ColumnData, ColumnValues, Field,
    PathSegment, SourceValue,
};

/// Builder for a `List<T>` column.
#[derive(Debug, Clone)]
pub struct ListBuilder {
    field: Field,
    offsets: Vec<i32>,
    validity: Vec<bool>,
    values: Box<ColumnBuilder>, // child builder
}

impl ListBuilder {
    pub fn new(field: Field, child: ColumnBuilder, rows_hint: usize) -> Self {
        let mut offsets = Vec::with_capacity(rows_hint + 1);
        offsets.push(0);
        Self {
            field,
            offsets,
            validity: Vec::with_capacity(rows_hint),
            values: Box::new(child),
        }
    }

    pub fn len(&self) -> usize {
        self.validity.len()
    }

    pub fn append_null(&mut self) {
        // For null lists, we push the same offset (empty list)
        let last = *self.offsets.last().unwrap();
        self.offsets.push(last);
        self.validity.push(false);
        // No need to append to child values since it's an empty list
    }

    pub fn append_raw<P: CoercePolicy>(
        &mut self, policy: &P, raw: SourceValue,
    ) -> Result<(), CoerceError> {
        match raw {
            SourceValue::Null => {
                self.append_null();
                Ok(())
            }
            SourceValue::List(items) => {
                let mut count = 0_i32;
                for (idx, item) in items.into_iter().enumerate() {
                    self.values
                        .append_raw(policy, item)
                        .map_err(|e| e.at(PathSegment::Index(idx)))?;
                    count += 1;
                }
                let last = *self.offsets.last().unwrap();
                self.offsets.push(last + count);
                self.validity.push(true);
                Ok(())
            }
            other => Err(CoerceError::mismatch("list", &other)),
        }
    }

    pub fn finish(&mut self) -> ColumnData {
        let offset_len = self.offsets.len();
        let offsets = std::mem::replace(&mut self.offsets, Vec::with_capacity(offset_len));
        self.offsets.push(0);
        let validity_len = self.validity.len();
        let validity = std::mem::replace(&mut self.validity, Vec::with_capacity(validity_len));
        ColumnData {
            kind: ColumnValues::List {
                field: self.field.clone(),
                offsets,
                values: Box::new(self.values.finish()),
            },
            validity,
        }
    }
}

/// Builder for a struct column.
///
/// Stores child columns in *schema* order; field names only exist here so we
/// can route row values correctly. The final `BufferKind::Struct` doesn't need names.
#[derive(Debug, Clone)]
pub struct StructBuilder {
    columns: Vec<ColumnBinding>,
    validity: Vec<bool>,
}

impl StructBuilder {
    pub fn new(columns: Vec<ColumnBinding>, rows_hint: usize) -> Self {
        Self {
            columns,
            validity: Vec::with_capacity(rows_hint),
        }
    }

    pub fn len(&self) -> usize {
        self.validity.len()
    }

    pub fn append_null(&mut self) {
        self.validity.push(false);
        // CRITICAL: Keep children in sync!
        for col in &mut self.columns {
            col.builder.append_null();
        }
    }

    pub fn append_raw<P: CoercePolicy>(
        &mut self, policy: &P, raw: SourceValue,
    ) -> Result<(), CoerceError> {
        match raw {
            SourceValue::Null => {
                self.append_null();
                Ok(())
            }
            SourceValue::Struct(entries) => {
                for col in &mut self.columns {
                    let name = &col.field.name;
                    let raw_field = entries.get(name.as_str()).unwrap_or(&SourceValue::Null);
                    col.builder
                        .append_raw(policy, raw_field.clone())
                        .map_err(|e| e.at(PathSegment::Field(name.clone())))?;
                }

                // mark struct itself valid
                self.validity.push(true);
                Ok(())
            }
            other => Err(CoerceError::mismatch("struct", &other)),
        }
    }

    pub fn finish(&mut self) -> ColumnData {
        let validity_len = self.validity.len();
        let validity = std::mem::replace(&mut self.validity, Vec::with_capacity(validity_len));
        let fields = self
            .columns
            .iter_mut()
            .map(|c| (c.field.clone(), c.builder.finish()))
            .collect();
        ColumnData {
            kind: ColumnValues::Struct { fields },
            validity,
        }
    }
}
