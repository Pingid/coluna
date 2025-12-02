const MILLIS_PER_DAY: i64 = 86_400_000;
const NANOS_PER_DAY: i64 = 86_400_000_000_000;

const NANOS_PER_SEC: i64 = 1_000_000_000;
const NANOS_PER_MILLI: i64 = 1_000_000;
const NANOS_PER_MICRO: i64 = 1_000;

/// Timestamp in nanoseconds since Unix epoch.
#[derive(Debug, Clone, Copy, Default)]
pub struct TemporalNano(pub i64);

impl From<i64> for TemporalNano {
    fn from(nanos: i64) -> Self {
        TemporalNano(nanos)
    }
}

impl TemporalNano {
    pub fn from_infer(v: i64) -> Self {
        let abs = v.abs();
        // < 1e11 → probably seconds
        if abs < 100_000_000_000 {
            return Self::from_secs(v);
        }
        // < 1e14 → probably millis
        if abs < 100_000_000_000_000 {
            return Self::from_millis(v);
        }
        // < 1e17 → probably micros
        if abs < 100_000_000_000_000_000 {
            return Self(v * NANOS_PER_MICRO);
        }

        // Otherwise assume nanos
        Self(v)
    }

    pub fn from_millis(ms: i64) -> Self {
        Self(ms * NANOS_PER_MILLI)
    }

    pub fn from_secs(s: i64) -> Self {
        Self(s * NANOS_PER_SEC)
    }

    pub fn from_secs_f64(s: f64) -> Self {
        Self((s * NANOS_PER_SEC as f64) as i64)
    }

    pub fn as_secs(&self) -> i64 {
        self.0 / NANOS_PER_SEC
    }

    pub fn as_millis(&self) -> i64 {
        self.0 / NANOS_PER_MILLI
    }

    pub fn as_micros(&self) -> i64 {
        self.0 / NANOS_PER_MICRO
    }

    pub fn as_nanos(&self) -> i64 {
        self.0
    }

    pub fn to_date32(&self) -> i32 {
        let nanos = self.0;
        let mut days = nanos / NANOS_PER_DAY;
        if nanos < 0 && nanos % NANOS_PER_DAY != 0 {
            days -= 1;
        }
        days as i32
    }

    pub fn to_date64(&self) -> i64 {
        let nanos = self.0;
        let mut days = nanos / NANOS_PER_DAY;
        if nanos < 0 && nanos % NANOS_PER_DAY != 0 {
            days -= 1;
        }
        days * MILLIS_PER_DAY
    }

    pub fn to_time32_s(&self) -> i32 {
        (self.nanos_since_midnight() / NANOS_PER_SEC) as i32
    }

    pub fn to_time32_ms(&self) -> i32 {
        (self.nanos_since_midnight() / NANOS_PER_MILLI) as i32
    }

    pub fn to_time64_us(&self) -> i64 {
        self.nanos_since_midnight() / NANOS_PER_MICRO
    }

    pub fn to_time64_ns(&self) -> i64 {
        self.nanos_since_midnight()
    }

    fn nanos_since_midnight(&self) -> i64 {
        let nanos = self.0;
        let mut rem = nanos % NANOS_PER_DAY;
        if rem < 0 {
            rem += NANOS_PER_DAY;
        }
        rem
    }
}

/// Duration in nanoseconds.
#[derive(Debug, Clone, Copy, Default)]
pub struct DurationNano(pub i64);

impl From<i64> for DurationNano {
    fn from(nanos: i64) -> Self {
        DurationNano(nanos)
    }
}

impl DurationNano {
    pub fn from_millis(ms: i64) -> Self {
        Self(ms * NANOS_PER_MILLI)
    }

    pub fn from_secs(s: i64) -> Self {
        Self(s * NANOS_PER_SEC)
    }

    pub fn from_secs_f64(s: f64) -> Self {
        Self((s * NANOS_PER_SEC as f64) as i64)
    }

    pub fn as_secs(&self) -> i64 {
        self.0 / NANOS_PER_SEC
    }

    pub fn as_millis(&self) -> i64 {
        self.0 / NANOS_PER_MILLI
    }

    pub fn as_micros(&self) -> i64 {
        self.0 / NANOS_PER_MICRO
    }

    pub fn as_nanos(&self) -> i64 {
        self.0
    }
}
