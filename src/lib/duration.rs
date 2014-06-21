use libc::types::os::common::posix01::{timespec, timeval};
use std::num::{Bounded, pow};
use std::u64;
use std::io::{MemWriter, IoResult};
use std::fmt::{Show, Formatter, FormatError, WriteError};
use std::fmt::rt::AlignLeft;
use std::from_str::FromStr;
use tick;

/// An absolute amount of time, independent of time zones and calendars.
/// A duration can express the positive or negative difference between two
/// instants in time according to a particular clock.
#[deriving(PartialEq, PartialOrd, Ord, Eq,
           Clone, Zero, Default, Hash, Rand)]
pub struct Duration {
    ticks : i64
}

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
        // We accept strings of the format "PT<N>S", where <N> is a decimal
        // number, possibly negative, possibly using a comma, and the
        // remaining characters are case-insensitive.

        let minval: i64 = Bounded::min_value();
        let maxval: i64 = Bounded::max_value();

        let r = s;
        let r = match r.slice_shift_char() {
            (Some(c), r) if c == 'P' || c == 'p' => r,
            _ => return None
        };
        let r = match r.slice_shift_char() {
            (Some(c), r) if c == 'T' || c == 't' => r,
            _ => return None
        };

        let (negative, r) = match r.slice_shift_char() {
            (Some(c), rem) => if c == '-' { (true, rem) } else { (false, r) },
            _ => return None
        };

        let (sec_part, r) = {
            let len = match r.find(|c: char| !(c >= '0' && c <= '9')) {
                Some(i) => i, None => return None
            };
            if len == 0 {
                return None
            }
            let n = match from_str::<u64>(r.slice_to(len)) {
                Some(n) => n, None => return None
            };
            let n = match n.checked_mul(&(tick::SECOND as u64)) {
                Some(n) => n, None => return None
            };
            (n, r.slice_from(len))
        };

        let (tick_part, r) = match r.slice_shift_char() {
            (Some(c), rem) if c == '.' || c == ',' => {
                let len = match rem.find(|c: char| !(c >= '0' && c <= '9')) {
                    Some(i) => i, None => return None
                };
                if len == 0 {
                    return None
                }
                let tick_part = if len <= 7 {
                    match from_str::<u64>(rem.slice_to(len)) {
                        Some(n) => n * pow(10u64, 7 - len),
                        None => return None
                    }
                } else {
                    let n = match from_str::<u64>(rem.slice_to(7)) {
                        Some(n) => n, None => return None
                    };
                    if rem.char_at(7) == '5' &&
                        rem.slice(8, len).chars().all(|c| c == '0') {
                        n + (n & 1)
                    } else if rem.char_at(7) >= '5' {
                        n + 1
                    } else {
                        n
                    }
                };
                (tick_part, rem.slice_from(len))
            }
            _ => (0u64, r)
        };

        match r.slice_shift_char() {
            (Some(c), r) if (c == 'S' || c == 's') && r.is_empty() => {}
            _ => return None
        }

        let ticks = match sec_part.checked_add(&tick_part) {
            Some(n) => n, None => return None
        };

        if negative {
            if ticks > minval as u64 {
                None
            } else {
                Some(Duration { ticks: -(ticks as i64) })
            }
        } else {
            if ticks > maxval as u64 {
                None
            } else {
                Some(Duration { ticks: ticks as i64 })
            }
        }
    }
}

impl Show for Duration {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        // Uses the ISO-8601 format for durations.
        // See: http://en.wikipedia.org/wiki/ISO_8601#Durations
        let datavec = match self.to_utf8_io(f.precision) {
            Ok(x) => x,
            Err(_) => return Err(WriteError)
        };
        let data = datavec.as_slice();
        let padding = match f.width {
            Some(width) => {
                let sz = data.len();
                if width > sz { width - sz } else { 0 }
            }
            None => 0
        };

        if padding == 0 {
            return f.write(data)
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
        try!(w.write_str("PT"))
        let (negative, mag) = if self.ticks >= 0 {
            (false, self.ticks as u64)
        } else {
            (true, -self.ticks as u64)
        };
        let (significand, dotpos, zeroes) = match precision {
            None => {
                let mut significand: u64 = mag;
                let mut dotpos: uint = 7;
                while dotpos > 0 && (significand % 10) == 0 {
                    significand /= 10;
                    dotpos -= 1;
                }
                (significand, dotpos, 0u)
            }
            Some(prec) => {
                if prec < 7 {
                    let dotpos = if prec > 0 { prec } else { 0 };
                    let x = pow(10u64, 7 - dotpos);
                    let mut ival = mag / x;
                    let rem = mag % x;
                    if rem > x / 2 || (rem == x / 2 && (ival & 1) == 1) {
                        ival += 1;
                    }
                    (ival, dotpos, 0u)
                } else {
                    (mag, 7, prec - 7)
                }
            }
        };
        if negative && significand > 0 {
            try!(w.write_char('-'));
        }
        try!(u64::to_str_bytes(significand, 10, |v| {
            if v.len() <= dotpos {
                try!(w.write_str("0.0000000".slice_to(2 + dotpos - v.len())));
                try!(w.write(v));
            } else {
                try!(w.write(v.slice_to(v.len() - dotpos)));
                if dotpos > 0 {
                    try!(w.write_char('.'));
                    try!(w.write(v.slice_from(v.len() - dotpos)));
                }
            }
            for _ in range(0, zeroes) {
                try!(w.write_char('0'));
            }
            Ok(())
        }));
        try!(w.write_char('S'));
        Ok(w.unwrap())
    }

    /// Convert from microseconds to a duration.  No overflow protection.
    pub fn from_microseconds(n: i64) -> Duration {
        Duration { ticks: tick::MICROSECOND * n }
    }

    /// Convert from milliseconds to a duration.  No overflow protection.
    pub fn from_milliseconds(n: i64) -> Duration {
        Duration { ticks: tick::MILLISECOND * n }
    }

    /// Convert from seconds to a duration.  No overflow protection.
    pub fn from_seconds(n: i64) -> Duration {
        Duration { ticks: tick::SECOND * n }
    }

    /// Convert from standard minutes to a duration.  No overflow protection.
    pub fn from_minutes(n: i64) -> Duration {
        Duration { ticks: tick::MINUTE * n }
    }

    /// Convert from standard hours to a duration.  No overflow protection.
    pub fn from_hours(n: i64) -> Duration {
        Duration { ticks: tick::HOUR * n }
    }

    /// Convert from standard days to a duration.  No overflow protection.
    pub fn from_standard_days(n: i64) -> Duration {
        Duration { ticks: tick::DAY * n }
    }

    /// Convert from standard weeks to a duration.  No overflow protection.
    pub fn from_standard_weeks(n: i64) -> Duration {
        Duration { ticks: tick::DAY * 7 * n }
    }

    /// Convert to whole seconds, with rounding.
    pub fn to_seconds(&self) -> i64 {
        tick::to_sec(self.ticks)
    }

    /// Convert to whole milliseconds, with rounding.
    pub fn to_milliseconds(&self) -> i64 {
        tick::to_msec(self.ticks)
    }

    /// Convert to whole microseconds, with rounding.
    pub fn to_microseconds(&self) -> i64 {
        tick::to_usec(self.ticks)
    }

    /// Convert to a POSIX timespec structure.
    pub fn to_timespec(&self) -> timespec {
        let (sec, nsec) = tick::to_sec_nsec(self.ticks);
        timespec { tv_sec: sec, tv_nsec: nsec as i64 }
    }

    /// Convert to a POSIX timeval structure, with rounding.
    pub fn to_timeval(&self) -> timeval {
        let (sec, usec) = tick::to_sec_usec(self.ticks);
        timeval { tv_sec: sec, tv_usec: usec as i64 }
    }
}

#[cfg(test)]
fn test_format_1(d: i64, s: &str) -> () {
    let out = format!("{}", Duration { ticks: d });
    if out.as_slice() != s {
        fail!("input: {}, expected: '{}', output: '{}'",
              d, s, out);
    }
}

#[test]
fn test_format() {
    test_format_1(0, "PT0S");
    test_format_1(1, "PT0.0000001S");
    test_format_1(-1, "PT-0.0000001S");
    test_format_1(10, "PT0.000001S");
    test_format_1(-10, "PT-0.000001S");
    test_format_1(100, "PT0.00001S");
    test_format_1(-100, "PT-0.00001S");
    test_format_1(1000, "PT0.0001S");
    test_format_1(-1000, "PT-0.0001S");
    test_format_1(10000, "PT0.001S");
    test_format_1(-10000, "PT-0.001S");
    test_format_1(100000, "PT0.01S");
    test_format_1(-100000, "PT-0.01S");
    test_format_1(1000000, "PT0.1S");
    test_format_1(-1000000, "PT-0.1S");

    test_format_1(tick::SECOND, "PT1S");
    test_format_1(-tick::SECOND, "PT-1S");

    test_format_1(tick::MINUTE, "PT60S");
    test_format_1(-tick::MINUTE, "PT-60S");

    test_format_1(tick::HOUR, "PT3600S");
    test_format_1(-tick::HOUR, "PT-3600S");

    test_format_1(tick::STANDARD_DAY, "PT86400S");
    test_format_1(-tick::STANDARD_DAY, "PT-86400S");

    test_format_1(tick::STANDARD_WEEK, "PT604800S");
    test_format_1(-tick::STANDARD_WEEK, "PT-604800S");

    test_format_1(Bounded::max_value(), "PT922337203685.4775807S");
    test_format_1(Bounded::min_value(), "PT-922337203685.4775808S");
}

#[cfg(test)]
fn test_roundtrip_1(n: Duration) {
    let s = format!("{}", n);
    match from_str::<Duration>(s.as_slice()) {
        None => fail!("cannot parse: {}", s),
        Some(m) => assert_eq!(n, m),
    };
}

#[test]
fn test_roundtrip() {
    test_roundtrip_1(Duration { ticks: 0 });
    test_roundtrip_1(Duration { ticks: 1 });
    test_roundtrip_1(Duration { ticks: -1 });
    test_roundtrip_1(Duration { ticks: 9999999 });
    test_roundtrip_1(Duration { ticks: -9999999 });
    test_roundtrip_1(Duration { ticks: 10000000 });
    test_roundtrip_1(Duration { ticks: -10000000 });
    test_roundtrip_1(Duration { ticks: 1234567890 });
    test_roundtrip_1(Duration { ticks: -1234567890 });
    test_roundtrip_1(Bounded::min_value());
    test_roundtrip_1(Bounded::max_value());
}

#[cfg(test)]
fn test_rounding_1(prec: uint, d: i64, s: &str) {
    let out = format!("{:.*}", prec, Duration { ticks: d });
    if out.as_slice() != s {
        fail!("precision: {}, input: {}, expected: '{}', output: '{}'",
              prec, d, s, out);
    }
}

#[test]
fn test_rounding() {
    test_rounding_1(0, 0, "PT0S");
    test_rounding_1(1, 0, "PT0.0S");
    test_rounding_1(2, 0, "PT0.00S");
    test_rounding_1(3, 0, "PT0.000S");
    test_rounding_1(4, 0, "PT0.0000S");
    test_rounding_1(5, 0, "PT0.00000S");
    test_rounding_1(6, 0, "PT0.000000S");
    test_rounding_1(7, 0, "PT0.0000000S");
    test_rounding_1(8, 0, "PT0.00000000S");
    test_rounding_1(9, 0, "PT0.000000000S");

    test_rounding_1(5,  50, "PT0.00000S");
    test_rounding_1(5,  51, "PT0.00001S");
    test_rounding_1(5, 149, "PT0.00001S");
    test_rounding_1(5, 150, "PT0.00002S");

    test_rounding_1(3, 59984999, "PT5.998S");
    test_rounding_1(3, 59985000, "PT5.998S");
    test_rounding_1(3, 59994999, "PT5.999S");
    test_rounding_1(3, 59995000, "PT6.000S");

    test_rounding_1(2, -50000, "PT0.00S");
    test_rounding_1(2, -50001, "PT-0.01S");
}

#[cfg(test)]
fn test_parsefail_1(s: &str) {
    match from_str::<Duration>(s) {
        None => (),
        Some(_) => fail!("input: '{}'", s),
    }
}

#[test]
fn test_parsefail() {
    test_parsefail_1("PT.S");
    test_parsefail_1("PT.0S");
    test_parsefail_1("PT0.S");
    test_parsefail_1("PTS");
    test_parsefail_1("PT-S");
    test_parsefail_1("PT-.S");
    test_parsefail_1("PT-0.S");
    test_parsefail_1("PT-.0S");
    test_parsefail_1("AT0S");
    test_parsefail_1("PU0S");
    test_parsefail_1("P0S");
    test_parsefail_1("T0S");
    test_parsefail_1("PT0");

    // Overflow causes parse failure
    test_parsefail_1("PT922337203685.4775808S");
    test_parsefail_1("PT-922337203685.4775809S");
    test_parsefail_1("PT922337203685.47758075S");
    test_parsefail_1("PT-922337203685.47758086S");
    test_parsefail_1("PT922337203686S");
    test_parsefail_1("PT-922337203686S");
    test_parsefail_1("PT100000000000000000000000000S");
    test_parsefail_1("PT-100000000000000000000000000S");
}

#[cfg(test)]
fn test_parse_1(s: &str, d: i64) {
    match from_str::<Duration>(s) {
        Some(r) => if r.ticks != d {
            fail!("input: '{}', expected: {}, output: {}",
                  s, d, r.ticks);
        },
        None => fail!("input: '{}' failed to parse")
    }
}

#[test]
fn test_parse() {
    test_parse_1("PT0S", 0);
    test_parse_1("PT-0S", 0);
    test_parse_1("pt0s", 0);
    test_parse_1("PT0.0000000S", 0);
    test_parse_1("PT0.000000000000000000S", 0);

    // Test rounding digits beyond the precision actually stored
    test_parse_1("PT0.00000005S", 0);
    test_parse_1("PT0.000000050S", 0);
    test_parse_1("PT0.0000000500000000000000000000000000000000000S", 0);
    test_parse_1("PT0.00000006S", 1);
    test_parse_1("PT0.000000051S", 1);
    test_parse_1("PT0.0000000500000000000000000000000000000000001S", 1);
    test_parse_1("PT0.00000014S", 1);
    test_parse_1("PT0.000000149999999999999999999999999999999999S", 1);
    test_parse_1("PT0.00000015S", 2);
}
