use std::fmt::{Formatter, FormatError};
use std::fmt::rt::AlignLeft;

/// Write a single field to a formatter with the selected padding and
/// alignment.  The field must already be converted to a slice of UTF-8 data.
pub fn write_field(f: &mut Formatter, data: &[u8]) -> Result<(), FormatError> {
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
