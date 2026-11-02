use std::io::Write;

use crate::error::Error;

pub fn print<W: Write>(value: i64, w: &mut W) -> Result<(), Error> {
    writeln!(w, "{value}").map_err(|err| Error::Report {
        message: value.to_string(),
        source: err,
    })
}
