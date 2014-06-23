/*
    ISO-8601 calendar conversion functions

    Uses the "chronological Julian day", which for this library, defines
    2000-01-01 to be day 0.  Converts between CJD and the ISO-8601 calendar.
    This module is internal because it just does the math and works on
    untyped values.  There is no error checking here.
*/

use std::num::div_rem;
use div_mod::div_mod;

static EPOCH_LEN: int = 146097;
static MONTHDAY: [int, ..12] = [
    0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334
];

/// Convert an ISO 8601 date to a chronological Julian day.
pub fn to_cjd(year: int, month: int, day: int) -> int {
    let y = year - 2000;
    let (b, a) = div_mod(y, 400);
    let mut leap = 1 + a / 4 - a / 100;
    if month <= 2 && (a % 4) == 0 && ((a % 100) != 0 || a == 0) {
        leap -= 1;
    }
    146097 * b + 365 * a + MONTHDAY[(month - 1) as uint] + day - 1 + leap
}

/// Convert a chronological Julian day to an ISO 8601 date.
pub fn from_cjd(cjd: int) -> (int, int, int) {
    let (t1, d) = div_mod(cjd, EPOCH_LEN);
    let (t2, d) = div_rem(d - 1, 36524);
    let (t3, d) = div_rem(d + 1, 1461);
    let (t4, d) = div_rem(d - 1, 365);
    let y = 2000 + t1 * 400 + t2 * 100 + t3 * 4 + t4;

    let mut m = d / 29 + 1;
    if m > 12 || (m > 1 && d < MONTHDAY[(m - 1) as uint]) {
        m -= 1;
    }

    let mut d = d - MONTHDAY[(m - 1) as uint] + 1;
    if t4 == 0 && (t3 != 0 || t2 == 0) && m <= 2 {
        d += 1;
        if d == 32 {
            m = 2;
            d = 1;
        }
    }

    (y, m, d)
}

#[test]
fn cjd_point() {
    fn test(cjd: int, y: int, m: int, d: int) {
        let out_cjd = to_cjd(y, m, d);
        if out_cjd != cjd {
            fail!("{:04d}-{:02d}-{:02d}: expected {}, got {}",
                  y, m, d, cjd, out_cjd);
        }

        let (y2, m2, d2) = from_cjd(cjd);
        if y != y2 || m != m2 || d != d2 {
            fail!("{}: expected {:04d}-{:02d}-{:02d}, got {:04d}-{:02d}-{:02d}",
                  cjd, y, m, d, y2, m2, d2);
        }
    }

    test(0, 2000, 1, 1);
    test(1, 2000, 1, 2);
    test(-1, 1999, 12, 31);
    test(146097, 2400, 1, 1);
    test(-146097, 1600, 1, 1);
}

#[test]
fn cjd_range() {
    static MONTHS: [int, ..12] = [
        31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31
    ];

    fn test_range(start: int, end: int) {
        let mut y = start;
        let mut m = 1;
        let mut d = 1;
        let mut last_cjd = to_cjd(y, m, d) - 1;
        while y < end {
            let cjd = to_cjd(y, m, d);
            if cjd != last_cjd + 1 {
                fail!("Nonconsecutive date.");
            }
            last_cjd = cjd;
            let (y2, m2, d2) = from_cjd(cjd);
            if y != y2 || m != m2 || d != d2 {
                fail!("Bad conversion: {:04d}-{:02d}-{:02d} to {} to {:04d}-{:02d}-{:02d}",
                      y, m, d, cjd, y2, m2, d2);
            }
            d += 1;
            if d > MONTHS[(m - 1) as uint] {
                let is_leap = m == 2 && d == 29 && (y % 4) == 0 &&
                              ((y % 100) != 0 || (y % 400) == 0);
                if !is_leap {
                    m += 1;
                    d = 1;
                    if m >= 13 {
                        m = 1;
                        y += 1;
                    }
                }
            }
        }
    }
    
    test_range(1596, 2404);
}
