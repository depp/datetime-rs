use libc::types::os::common::posix01::{timespec, timeval};
use std::num::Bounded;
use std::io::{MemWriter, IoResult};
use std::fmt::{Show, Formatter, FormatError, WriteError};
use tick;
use fmtutil;
use calendar_iso8601;
use duration::Duration;
use div_mod::div_mod;
use std::num::div_rem;

/// An absolute moment in time, independent of time zones and calendars.
/// This uses the default time scale, which does not account for leap seconds.
#[deriving(PartialEq, PartialOrd, Ord, Eq,
           Clone, Hash, Rand)]
pub struct Instant {
    pub ticks : i64,
}

/// The unix epoch, in days
static UNIX_EPOCH_DAY: i64 = -10957;

/// This library's epoch, relative to the Unix epoch.
static EPOCH_UNIX_SECOND: i64 = 10957 * 86400;

/// The Unix epoch: January 1, 1970.
pub static UNIX_EPOCH: Instant = Instant {
    ticks: tick::DAY * UNIX_EPOCH_DAY
};

impl Add<Duration, Instant> for Instant {
    fn add(&self, rhs: &Duration) -> Instant {
        Instant { ticks: self.ticks + rhs.ticks }
    }
}

impl Sub<Duration, Instant> for Instant {
    fn sub(&self, rhs: &Duration) -> Instant {
        Instant { ticks: self.ticks - rhs.ticks }
    }
}

impl Bounded for Instant {
    fn min_value() -> Instant {
        Instant { ticks: Bounded::min_value() }
    }

    fn max_value() -> Instant {
        Instant { ticks: Bounded::max_value() }
    }
}

impl Show for Instant {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        let datavec = match self.to_utf8_io() {
            Ok(x) => x,
            Err(_) => return Err(WriteError)
        };
        fmtutil::write_field(f, datavec.as_slice())
    }
}

impl Instant {
    fn to_utf8_io(&self) -> IoResult<Vec<u8>> {
        let mut w = MemWriter::with_capacity(32);
        let (cjd, tickrem) = div_mod(self.ticks, tick::DAY);
        let (y, m, d) = calendar_iso8601::from_cjd(cjd as int);
        let (ss, _) = div_rem(tickrem, tick::SECOND);
        let (mm, ss) = div_rem(ss as int, 60);
        let (hh, mm) = div_rem(mm, 60);
        try!(write!(w, "{:04d}-{:02d}-{:02d}T{:02d}:{:02d}:{:02d}Z",
                    y, m, d, hh, mm, ss));
        Ok(w.unwrap())
    }

    /// Convert from a POSIX timeval structure.  The input must measure
    /// time since the POSIX epoch, using the POSIX time scale.
    pub fn from_timespec(value: &timespec) -> Option<Duration> {
        let sec = match value.tv_sec.checked_add(&-EPOCH_UNIX_SECOND) {
            Some(n) => n, None => return None
        };
        tick::from_sec_nsec(sec, value.tv_nsec)
            .map(|n| Duration { ticks: n })
    }

    /// Convert from a POSIX timespec structure.  The input must measure
    /// time since the POSIX epoch, using the POSIX time scale.
    pub fn from_timesval(value: &timeval) -> Option<Duration> {
        let sec = match value.tv_sec.checked_add(&-EPOCH_UNIX_SECOND) {
            Some(n) => n, None => return None
        };
        tick::from_sec_usec(sec, value.tv_usec as i64)
            .map(|n| Duration { ticks: n })
    }

    /// Convert to a POSIX timeval structure.  The resulting structure
    /// will measure time since the POSIX epoch, using the POSIX time scale.
    pub fn to_timeval(&self) -> timeval {
        let (sec, usec) = tick::to_sec_usec(self.ticks);
        timeval { tv_sec: sec + EPOCH_UNIX_SECOND, tv_usec: usec }
    }

    /// Convert to a POSIX timespec structure.  The resulting structure
    /// will measure time since the POSIX epoch, using the POSIX time scale.
    pub fn to_timespec(&self) -> timespec {
        let (sec, nsec) = tick::to_sec_nsec(self.ticks);
        timespec { tv_sec: sec + EPOCH_UNIX_SECOND, tv_nsec: nsec as i64 }
    }
}

#[test]
fn format() {
    fn test(expected: &str, ticks: i64) {
        let output = format!("{}", Instant { ticks: ticks });
        if output.as_slice() != expected {
            fail!("ticks: {}, expected: '{}', output: '{}'",
                  ticks, expected, output);
        }
    }

    test("2000-01-01T00:00:00Z", 0);
    test("2000-01-01T00:00:01Z", tick::SECOND);
    test("1999-12-31T23:59:59Z", -tick::SECOND);
    test("2000-01-01T00:01:00Z", tick::MINUTE);
    test("1999-12-31T23:59:00Z", -tick::MINUTE);
    test("2000-01-01T01:00:00Z", tick::HOUR);
    test("1999-12-31T23:00:00Z", -tick::HOUR);
    test("2000-01-02T00:00:00Z", tick::DAY);
    test("1999-12-31T00:00:00Z", -tick::DAY);
}
