#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_id: TypeId,
    pub nullable: bool,
}

impl Field {
    pub fn new(name: String, type_id: TypeId, nullable: bool) -> Self {
        Self {
            name,
            type_id,
            nullable,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeId {
    Null,
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Timestamp,
    Duration,
    Decimal128,
    Utf8,
    LargeUtf8,
    Binary,
    LargeBinary,
    FixedSizeBinary,
    List(Box<TypeId>),
    Struct(Vec<TypeId>),
}

#[macro_export]
macro_rules! access_types {
    ($f:ident) => {
        $f!(
            (TypeId::Null, get_null),
            (TypeId::Boolean, get_boolean),
            (TypeId::Int8, get_int8),
            (TypeId::Int16, get_int16),
            (TypeId::Int32, get_int32),
            (TypeId::Int64, get_int64),
            (TypeId::UInt8, get_uint8),
            (TypeId::UInt16, get_uint16),
            (TypeId::UInt32, get_uint32),
            (TypeId::UInt64, get_uint64),
            (TypeId::Float32, get_float32),
            (TypeId::Float64, get_float64),
            (TypeId::Timestamp, get_timestamp),
            (TypeId::Duration, get_duration),
            (TypeId::Decimal128, get_decimal128),
            (TypeId::Utf8, get_utf8),
            (TypeId::LargeUtf8, get_large_utf8),
            (TypeId::Binary, get_binary),
            (TypeId::LargeBinary, get_large_binary),
            (TypeId::FixedSizeBinary, get_fixed_size_binary),
            (TypeId::List(_), get_list),
            (TypeId::Struct(_), get_struct),
        );
    };
}
