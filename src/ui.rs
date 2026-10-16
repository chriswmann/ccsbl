use std::io::Write;

use crate::error::RuntimeError;

pub fn print<W: Write>(value: i64, w: &mut W) -> Result<(), RuntimeError> {
    writeln!(w, "{value}").map_err(|err| RuntimeError::Report {
        message: value.to_string(),
        source: err,
    })
}
