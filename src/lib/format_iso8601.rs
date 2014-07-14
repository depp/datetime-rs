use std::num::Bounded;

#[deriving(PartialEq, PartialOrd, Ord, Eq, Clone, Hash, Rand)]
pub enum Date {
    Year(int),
    YearMonth(int, int),
    YearMonthDay(int, int, int),
    YearDay(int, int),
    YearWeek(int, int),
    YearWeekDay(int, int, int)
}

/// Read an integer from a bytestring.  Returns the integer, its length, and
/// the remainter of the string.
fn read_int<'a>(s: &'a [u8]) -> (int, uint, &'a [u8]) {
    let mut value: int = 0;
    let max: int = Bounded::max_value();
    for (i, x) in s.iter().enumerate() {
        let d = (*x as int) - ('0' as int);
        if d >= 0 && d <= 9 {
            value = match value.checked_mul(&10) {
                Some(n) => match n.checked_add(&d) {
                    Some(n) => n,
                    None => max
                },
                None => max
            };
        } else {
            return (value, i, s.slice_from(i));
        }
    }
    (value, s.len(), &[])
}

/// Parse a date in ISO 8601 format.
pub fn parse_date(s: &str) -> Option<Date> {
    let rem = s.as_bytes();
    if rem.is_empty() {
        return None;
    }
    let (negative, rem) = match rem[0] as char {
        '-' => (true, rem.slice_from(1)),
        '+' => (false, rem.slice_from(1)),
        _ => (false, rem)
    };
    let (year, n, rem) = read_int(rem);
    if n < 4 {
        return None;
    }
    let year = if negative { -year } else { year };
    Some(if rem.is_empty() {
        Year(year)
    } else {
        if rem[0] != '-' as u8 || rem.len() < 2 {
            return None;
        }
        if rem[1] == 'W' as u8 {
            let (week, n, rem) = read_int(rem.slice_from(2));
            if n != 2 || week < 1 || week > 53 {
                return None;
            }
            if rem.is_empty() {
                YearWeek(year, week)
            } else {
                if rem[0] != '-' as u8 {
                    return None;
                }
                let (day, n, rem) = read_int(rem.slice_from(1));
                if !rem.is_empty() || n != 1 || day < 1 || day > 7 {
                    return None;
                }
                YearWeekDay(year, week, day)
            }
        } else {
            let (value, n, rem) = read_int(rem.slice_from(1));
            if n == 2 {
                let month = value;
                if month < 1 || month > 12 {
                    return None;
                }
                if rem.is_empty() {
                    YearMonth(year, month)
                } else {
                    if rem[0] != '-' as u8 {
                        return None;
                    }
                    let (day, n, rem) = read_int(rem.slice_from(1));
                    if !rem.is_empty() || n != 2 || day < 1 || day > 31 {
                        return None;
                    }
                    YearMonthDay(year, month, day)
                }
            } else if n == 3 {
                let day = value;
                if !rem.is_empty() || day < 1 || day > 366 {
                    return None;
                }
                YearDay(year, day)
            } else {
                return None;
            }
        }
    })
}

#[test]
fn test_read_date() {
    fn test(s: &str, d: Date) {
        match parse_date(s) {
            None => fail!("Could not parse: {}", s),
            Some(x) => if x != d {
                fail!("Incorrect parse: {}", s);
            }
        }
    }
    
    test("2000", Year(2000));
    test("+2000", Year(2000));
    test("-10000", Year(-10000));
    test("+0000", Year(0));
    test("-0000", Year(0));
    test("2000-12", YearMonth(2000, 12));
    test("1492-07-31", YearMonthDay(1492, 7, 31));
    test("1950-001", YearDay(1950, 1));
    test("1999-365", YearDay(1999, 365));
    test("1995-W01", YearWeek(1995, 1));
    test("2007-W44-7", YearWeekDay(2007, 44, 7));
}
