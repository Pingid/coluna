-- Demo table with all PostgreSQL data types

-- Custom enum type
CREATE TYPE mood AS ENUM ('happy', 'sad', 'neutral');
CREATE TYPE status AS ENUM ('pending', 'active', 'completed', 'cancelled');

-- Composite type
CREATE TYPE address AS (
    street TEXT,
    city TEXT,
    zip VARCHAR(10)
);

CREATE TABLE demo_types (
    id SERIAL PRIMARY KEY,

    -- Numeric types
    col_smallint SMALLINT,
    col_integer INTEGER,
    col_bigint BIGINT,
    col_decimal DECIMAL(10, 2),
    col_numeric NUMERIC(15, 5),
    col_real REAL,
    col_double DOUBLE PRECISION,
    col_smallserial SMALLSERIAL,
    col_bigserial BIGSERIAL,

    -- Monetary
    col_money MONEY,

    -- Character types
    col_char CHAR(10),
    col_varchar VARCHAR(255),
    col_text TEXT,

    -- Binary
    col_bytea BYTEA,

    -- Date/Time types
    col_date DATE,
    col_time TIME,
    col_time_tz TIME WITH TIME ZONE,
    col_timestamp TIMESTAMP,
    col_timestamp_tz TIMESTAMP WITH TIME ZONE,
    col_interval INTERVAL,

    -- Boolean
    col_boolean BOOLEAN,

    -- Geometric types
    col_point POINT,
    col_line LINE,
    col_lseg LSEG,
    col_box BOX,
    col_path PATH,
    col_polygon POLYGON,
    col_circle CIRCLE,

    -- Network types
    col_cidr CIDR,
    col_inet INET,
    col_macaddr MACADDR,
    col_macaddr8 MACADDR8,

    -- Bit string types
    col_bit BIT(8),
    col_bit_varying BIT VARYING(64),

    -- Text search types
    col_tsvector TSVECTOR,
    col_tsquery TSQUERY,

    -- UUID
    col_uuid UUID,

    -- XML
    col_xml XML,

    -- JSON types
    col_json JSON,
    col_jsonb JSONB,

    -- Array types
    col_int_array INTEGER[],
    col_text_array TEXT[],
    col_float_array DOUBLE PRECISION[],
    col_bool_array BOOLEAN[],

    -- Range types
    col_int4range INT4RANGE,
    col_int8range INT8RANGE,
    col_numrange NUMRANGE,
    col_tsrange TSRANGE,
    col_tstzrange TSTZRANGE,
    col_daterange DATERANGE,

    -- Enum type
    col_mood mood,
    col_status status,

    -- Composite type
    col_address address,

    -- OID type
    col_oid OID,

    -- pg_lsn (Log Sequence Number)
    col_pg_lsn PG_LSN
);

-- Insert sample data
INSERT INTO demo_types (
    col_smallint, col_integer, col_bigint, col_decimal, col_numeric,
    col_real, col_double,
    col_money,
    col_char, col_varchar, col_text,
    col_bytea,
    col_date, col_time, col_time_tz, col_timestamp, col_timestamp_tz, col_interval,
    col_boolean,
    col_point, col_line, col_lseg, col_box, col_path, col_polygon, col_circle,
    col_cidr, col_inet, col_macaddr, col_macaddr8,
    col_bit, col_bit_varying,
    col_tsvector, col_tsquery,
    col_uuid,
    col_xml,
    col_json, col_jsonb,
    col_int_array, col_text_array, col_float_array, col_bool_array,
    col_int4range, col_int8range, col_numrange, col_tsrange, col_tstzrange, col_daterange,
    col_mood, col_status,
    col_address,
    col_oid, col_pg_lsn
) VALUES (
    -- Numeric
    32767, 2147483647, 9223372036854775807, 12345.67, 9999999.12345,
    3.14159, 3.141592653589793,
    -- Monetary
    '$1,234.56',
    -- Character
    'ABCDEFGHIJ', 'Hello, PostgreSQL!', 'This is a longer text field with more content.',
    -- Binary
    E'\\xDEADBEEF',
    -- Date/Time
    '2024-12-04', '14:30:00', '14:30:00+05:30', '2024-12-04 14:30:00', '2024-12-04 14:30:00+00', '1 year 2 months 3 days 4 hours',
    -- Boolean
    TRUE,
    -- Geometric
    '(1.5, 2.5)', '{1, 2, 3}', '[(0,0),(1,1)]', '((0,0),(1,1))', '[(0,0),(1,1),(2,0)]', '((0,0),(1,0),(1,1),(0,1))', '<(0,0),5>',
    -- Network
    '192.168.0.0/24', '192.168.1.100', '08:00:2b:01:02:03', '08:00:2b:01:02:03:04:05',
    -- Bit string
    B'10101010', B'1100110011',
    -- Text search
    'quick brown fox jumps', 'fox & dog',
    -- UUID
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    -- XML
    '<root><element>value</element></root>',
    -- JSON
    '{"key": "value", "number": 42}', '{"name": "test", "tags": ["a", "b"]}',
    -- Arrays
    ARRAY[1, 2, 3, 4, 5], ARRAY['one', 'two', 'three'], ARRAY[1.1, 2.2, 3.3], ARRAY[true, false, true],
    -- Ranges
    '[1, 10)', '[100, 1000)', '[1.5, 9.5]', '[2024-01-01 00:00, 2024-12-31 23:59)', '[2024-01-01 00:00+00, 2024-12-31 23:59+00)', '[2024-01-01, 2024-12-31]',
    -- Enum
    'happy', 'active',
    -- Composite
    ROW('123 Main St', 'Springfield', '12345'),
    -- OID
    12345,
    -- pg_lsn
    '16/B374D848'
);

-- Insert a row with NULLs to test null handling
INSERT INTO demo_types (col_text) VALUES ('Row with mostly NULLs');

