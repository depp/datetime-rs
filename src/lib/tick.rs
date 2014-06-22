/*
 * Note: the terminology "standard minute" acknowledges that minutes may
 * have more or less than 60 seconds if there is a leap second during
 * that minute.
 */

/// The number of nanoseconds in a tick.
static PER_NANOSECOND: i64 = 100;
/// The number of ticks in a microsecond.
pub static MICROSECOND: i64 = 10;
/// The number of ticks in a millisecond.
pub static MILLISECOND: i64 = 1000 * MICROSECOND;
/// The number of ticks in a second.
pub static SECOND: i64 = 1000 * MILLISECOND;
/// The number of ticks in a standard minute.
pub static MINUTE: i64 = 60 * SECOND;
/// The number of ticks in a standard hour.
pub static HOUR: i64 = 60 * MINUTE;
/// The number of ticks in a standard day.
pub static DAY: i64 = 24 * HOUR;

/// Convert ticks to a larger unit, rounding to the nearest even integer.
#[inline]
fn to_larger_unit(ticks: i64, unit: i64) -> i64 {
    let mut total = ticks / unit;
    let mut rem = ticks % unit;
    if rem < 0 {
        total -= 1;
        rem += unit;
    }
    if rem > unit / 2 ||
        (rem == unit / 2 && (total & 1) != 0) {
        total += 1
    }
    total
}

/// Convert ticks to whole seconds, with rounding.
pub fn to_sec(ticks: i64) -> i64 {
    to_larger_unit(ticks, SECOND)
}

/// Convert ticks to whole milliseconds, with rounding.
pub fn to_msec(ticks: i64) -> i64 {
    to_larger_unit(ticks, MILLISECOND)
}

/// Convert ticks to whole microseconds, with rounding.
pub fn to_usec(ticks: i64) -> i64 {
    to_larger_unit(ticks, MICROSECOND)
}

/// Convert ticks to seconds and fractional microseconds, with rounding.
pub fn to_sec_usec(ticks: i64) -> (i64, i32) {
    let total_usec = to_larger_unit(ticks, MICROSECOND);
    let mut sec = total_usec / 1000000;
    let mut usec = (total_usec % 1000000) as i32;
    if usec < 0 {
        sec -= 1;
        usec += 1000000;
    }
    (sec, usec)
}

/// Convert ticks to seconds and fractional nanoseconds.
pub fn to_sec_nsec(ticks: i64) -> (i64, i32) {
    let mut sec = ticks / SECOND;
    let mut rem = (ticks % SECOND) as i32;
    if rem < 0 {
        sec -= 1;
        rem += SECOND as i32;
    }
    (sec, rem * 100)
}

/// Convert from larger units to ticks.  The units can't be larger than
/// about 2^30.
fn from_larger_units(v1: i64, v2: i64, u1: i64, u2: i64) -> Option<i64> {
    // Why is this complicated?  Because intermediate results may overflow
    // even if the final result does not.
    let hi: i64 = (v1 >> 32) * u1 + (v2 >> 32) * u2;
    let lo: i64 = (v1 & 0xffffffff) * u1 + (v2 & 0xffffffff) * u2;
    let hi: i64 = hi + (lo >> 32);
    let lo: i64 = lo & 0xffffffff;
    if hi != (hi as i32) as i64 {
        return None;
    }
    Some(hi | (lo << 32))
}

/// Convert from seconds and microseconds to ticks.  Returns None on
/// overflow.
pub fn from_sec_usec(sec: i64, usec: i64) -> Option<i64> {
    from_larger_units(sec, usec, SECOND, MICROSECOND)
}

/// Convert from seconds and nanoseconds to ticks.  Returns None on
/// overflow.  Rounds to the nearest integral number of ticks.
pub fn from_sec_nsec(sec: i64, nsec: i64) -> Option<i64> {
    from_larger_units(sec, to_larger_unit(nsec, PER_NANOSECOND),
                      SECOND, 1)
}
