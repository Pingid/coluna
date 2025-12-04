use arrow_lib::datatypes;

use crate::encoding::{
    Builder, ColumnBinding, ColumnBuilder, ColumnSpec, Field, FinishableStorage,
    FixedSizeBinaryStorage, ListBuilder, StructBuilder, TypeId, ValueStorage,
};

impl ColumnSpec for datatypes::Schema {
    fn build_columns(&self, rows: usize, value_hint: usize) -> Vec<ColumnBinding> {
        self.fields()
            .iter()
            .map(|f| {
                let (type_id, builder) = get_type_builder(f, rows, value_hint);
                ColumnBinding::new(
                    Field::new(f.name().to_string(), type_id, f.is_nullable()),
                    builder,
                )
            })
            .collect()
    }
}

fn get_type_builder(field: &datatypes::Field, rows: usize, vals: usize) -> (TypeId, ColumnBuilder) {
    use datatypes::DataType as DT;
    use {ColumnBuilder as B, TypeId as TI};

    match field.data_type() {
        DT::Null => (TI::Null, B::Null(builder(field, rows, vals))),
        DT::Boolean => (TI::Boolean, B::Boolean(builder(field, rows, vals))),
        DT::Int8 => (TI::Int8, B::Int8(builder(field, rows, vals))),
        DT::Int16 => (TI::Int16, B::Int16(builder(field, rows, vals))),
        DT::Int32 => (TI::Int32, B::Int32(builder(field, rows, vals))),
        DT::Int64 => (TI::Int64, B::Int64(builder(field, rows, vals))),
        DT::UInt8 => (TI::UInt8, B::UInt8(builder(field, rows, vals))),
        DT::UInt16 => (TI::UInt16, B::UInt16(builder(field, rows, vals))),
        DT::UInt32 => (TI::UInt32, B::UInt32(builder(field, rows, vals))),
        DT::UInt64 => (TI::UInt64, B::UInt64(builder(field, rows, vals))),
        DT::Float16 => (TI::Float32, B::Float32(builder(field, rows, vals))),
        DT::Float32 => (TI::Float32, B::Float32(builder(field, rows, vals))),
        DT::Float64 => (TI::Float64, B::Float64(builder(field, rows, vals))),
        DT::Timestamp(_, _tz) => (TI::Timestamp, B::Timestamp(builder(field, rows, vals))),
        DT::Date32 => (TI::Timestamp, B::Timestamp(builder(field, rows, vals))),
        DT::Date64 => (TI::Timestamp, B::Timestamp(builder(field, rows, vals))),
        DT::Time32(_) => (TI::Timestamp, B::Timestamp(builder(field, rows, vals))),
        DT::Time64(_) => (TI::Timestamp, B::Timestamp(builder(field, rows, vals))),
        DT::Duration(_) => (TI::Duration, B::Duration(builder(field, rows, vals))),
        DT::Interval(_) => todo!("Interval type not yet supported"),
        DT::Binary => (TI::Binary, B::Binary(builder(field, rows, vals))),
        DT::FixedSizeBinary(size) => (
            TI::FixedSizeBinary,
            B::FixedSizeBinary(Builder::new_with_capacity(
                FixedSizeBinaryStorage::with_capacity(*size, rows),
                field.is_nullable(),
                rows,
            )),
        ),
        DT::LargeBinary => (TI::LargeBinary, B::LargeBinary(builder(field, rows, vals))),
        DT::BinaryView => todo!("BinaryView type not yet supported"),
        DT::Utf8 => (TI::Utf8, B::Utf8(builder(field, rows, vals))),
        DT::LargeUtf8 => (TI::LargeUtf8, B::LargeUtf8(builder(field, rows, vals))),
        DT::Utf8View => todo!("Utf8View type not yet supported"),
        DT::List(list_field) => {
            let (type_id, builder) = get_type_builder(list_field, rows, vals);
            let field = Field::new(
                list_field.name().to_string(),
                type_id.clone(),
                list_field.is_nullable(),
            );
            (
                TI::List(Box::new(type_id)),
                B::List(ListBuilder::new(field, builder, rows)),
            )
        }
        DT::ListView(_) => todo!("ListView type not yet supported"),
        DT::FixedSizeList(_, _) => todo!("FixedSizeList type not yet supported"),
        DT::LargeList(_) => todo!("LargeList type not yet supported"),
        DT::LargeListView(_) => todo!("LargeListView type not yet supported"),
        DT::Struct(fields) => {
            let mut columns = Vec::new();
            let mut tps = Vec::new();
            for field in fields {
                let (type_id, builder) = get_type_builder(field, rows, vals);
                tps.push(type_id.clone());
                columns.push(ColumnBinding::new(
                    Field::new(field.name().to_string(), type_id, field.is_nullable()),
                    builder,
                ));
            }

            (
                TI::Struct(tps),
                B::Struct(StructBuilder::new(columns, rows)),
            )
        }
        DT::Union(_, _) => todo!("Union type not yet supported"),
        DT::Dictionary(_, _) => todo!("Dictionary type not yet supported"),
        #[cfg(not(feature = "arrow-54"))]
        DT::Decimal32(_, _) => todo!("Decimal32 type not yet supported"),
        #[cfg(not(feature = "arrow-54"))]
        DT::Decimal64(_, _) => todo!("Decimal64 type not yet supported"),
        DT::Decimal128(_, _) => (TI::Decimal128, B::Decimal128(builder(field, rows, vals))),
        DT::Decimal256(_, _) => todo!("Decimal256 type not yet supported"),
        DT::Map(_, _) => todo!("Map type not yet supported"),
        DT::RunEndEncoded(_, _) => todo!("RunEndEncoded type not yet supported"),
    }
}

fn builder<T: ValueStorage + FinishableStorage>(
    field: &datatypes::Field, rows: usize, value_hint: usize,
) -> Builder<T> {
    Builder::with_capacity(field.is_nullable(), rows, value_hint)
}
