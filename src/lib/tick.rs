/*
 * Note: the terminology "standard minute" acknowledges that minutes may
 * have more or less than 60 seconds if there is a leap second during
 * that minute.
 */

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
