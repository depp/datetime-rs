use std::num::Bounded;
use std::io::{MemWriter, BufWriter, IoResult};
use std::fmt::{Show, Formatter, FormatError, WriteError};
use std::fmt::rt::AlignLeft;
use std::from_str::FromStr;

/// An absolute amount of time, independent of time zones and calendars.
/// A duration can express the positive or negative difference between two
/// instants in time according to a particular clock.
#[deriving(PartialEq, PartialOrd, Ord, Eq,
           Clone, Zero, Default, Hash, Rand)]
pub struct Duration {
    ticks : i64
}

/// Duration of one microsecond.
pub static MICROSECOND: Duration = Duration { ticks: 10 };

/// Duration of one millisecond.
pub static MILLISECOND: Duration = Duration { ticks: 1000 * MICROSECOND.ticks };

/// Duration of one second.
pub static SECOND: Duration = Duration { ticks: 1000 * MILLISECOND.ticks };

/// Duration of one minute.
pub static MINUTE: Duration = Duration { ticks: 60 * SECOND.ticks };

/// Duration of one hour.
pub static HOUR: Duration = Duration { ticks: 60 * MINUTE.ticks };

/// Duration of one standard day (24 hours).
pub static STANDARD_DAY: Duration = Duration { ticks: 24 * HOUR.ticks };

/// Duration of one standard week (7 standard days).
pub static STANDARD_WEEK: Duration = Duration { ticks: 7 * STANDARD_DAY.ticks };

impl Add<Duration, Duration> for Duration {
    fn add(&self, rhs: &Duration) -> Duration {
        Duration { ticks: self.ticks + rhs.ticks }
    }
}

impl CheckedAdd for Duration {
    fn checked_add(&self, rhs: &Duration) -> Option<Duration> {
        self.ticks.checked_add(&rhs.ticks).map(|x| Duration { ticks: x })
    }
}

impl Sub<Duration, Duration> for Duration {
    fn sub(&self, rhs: &Duration) -> Duration {
        Duration { ticks: self.ticks - rhs.ticks }
    }
}

impl CheckedSub for Duration {
    fn checked_sub(&self, rhs: &Duration) -> Option<Duration> {
        self.ticks.checked_sub(&rhs.ticks).map(|x| Duration { ticks: x })
    }
}

impl Neg<Duration> for Duration {
    fn neg(&self) -> Duration {
        Duration { ticks: -self.ticks }
    }
}

impl Mul<i64, Duration> for Duration {
    fn mul(&self, rhs: &i64) -> Duration {
        Duration { ticks: rhs * self.ticks }
    }
}

impl Bounded for Duration {
    fn min_value() -> Duration {
        Duration { ticks: Bounded::min_value() }
    }

    fn max_value() -> Duration {
        Duration { ticks: Bounded::max_value() }
    }
}

impl FromStr for Duration {
    fn from_str(s: &str) -> Option<Duration> {
        None
    }
}

impl Show for Duration {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        // Uses the ISO-8601 format for durations.
        // See: http://en.wikipedia.org/wiki/ISO_8601#Durations
        let datavec = match self.to_utf8_io(f.precision) {
            Err(_) => return Err(WriteError),
            Ok(x) => x,
        };
        let data = datavec.as_slice();
        let padding = match f.width {
            None => 0,
            Some(width) => {
                let sz = data.len();
                if width > sz { width - sz } else { 0 }
            }
        };

        if padding == 0 {
            return f.write(data);
        }

        if f.align == AlignLeft {
            try!(f.write(data));
        }
        let mut fill = [0, ..4];
        let filllen = f.fill.encode_utf8(fill);
        let fill = fill.slice_to(filllen);
        for _ in range(0, padding) {
            try!(f.write(fill));
        }
        if f.align != AlignLeft {
            try!(f.write(data));
        }
        Ok(())
    }
}

impl Duration {
    /// Convert a duration to a UTF-8 vector, used to implement Show.
    /// The IoResult is a convenience so we can use try!().
    fn to_utf8_io(&self, precision: Option<uint>) -> IoResult<Vec<u8>> {
        let mut w = MemWriter::with_capacity(32);
        try!(w.write_str("PT"));
        let mag = if self.ticks >= 0 {
            self.ticks as u64
        } else {
            try!(w.write_char('-'));
            -self.ticks as u64
        };
        let secs = mag / (SECOND.ticks as u64);
        let ticks = mag % (SECOND.ticks as u64);
        try!(write!(&mut w, "{:u}", secs));
        match precision {
            Some(prec) => {
                if prec > 0 {
                    try!(w.write_char('.'));
                    let mut buf = [0u8, ..7];
                    {
                        let mut bw = BufWriter::new(buf);
                        try!(write!(&mut bw, "{:07u}", ticks));
                    }
                    if prec <= 7 {
                        try!(w.write(buf.slice(0, prec)));
                    } else {
                        try!(w.write(buf));
                        for _ in range(0, prec - 7) {
                            try!(w.write_char('0'));
                        }
                    }
                }
            },
            None => {
                if ticks > 0 {
                    let mut buf = [0u8, ..7];
                    {
                        let mut bw = BufWriter::new(buf);
                        try!(write!(&mut bw, "{:07u}", ticks));
                    }
                    let mut prec = 0;
                    for i in range(0, 7u) {
                        if buf[i] != '0' as u8 {
                            prec = i + 1;
                        }
                    }
                    if prec > 0 {
                        try!(w.write_char('.'));
                        try!(w.write(buf.slice(0, prec)));
                    }
                }
            }
        };
        Ok(w.unwrap())
    }

    /// Convert from microseconds to a duration.
    pub fn from_microseconds(n: i64) -> Duration {
        MICROSECOND * n
    }

    /// Convert from milliseconds to a duration.
    pub fn from_milliseconds(n: i64) -> Duration {
        MILLISECOND * n
    }

    /// Convert from seconds to a duration.
    pub fn from_seconds(n: i64) -> Duration {
        SECOND * n
    }

    /// Convert from minutes to a duration.
    pub fn from_minutes(n: i64) -> Duration {
        MINUTE * n
    }

    /// Convert from hours to a duration.
    pub fn from_hours(n: i64) -> Duration {
        HOUR * n
    }

    /// Convert from standard days to a duration.
    pub fn from_standard_days(n: i64) -> Duration {
        STANDARD_DAY * n
    }

    /// Convert from standard weeks to a duration.
    pub fn from_standard_weeks(n: i64) -> Duration {
        STANDARD_WEEK * n
    }
}
