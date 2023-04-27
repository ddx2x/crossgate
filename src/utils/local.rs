use anyhow::{Context, Result};
use std::fmt::Display;

pub struct Location {
    file: &'static str,
    line: u32,
    column: u32,
}

pub trait ErrorLocation<T, E> {
    fn location(self, loc: &'static Location) -> Result<T>;
}

impl<T, E> ErrorLocation<T, E> for Result<T, E>
where
    E: Display,
    Result<T, E>: Context<T, E>,
{
    fn location(self, loc: &'static Location) -> Result<T> {
        let msg = self.as_ref().err().map(ToString::to_string);
        self.with_context(|| {
            format!(
                "{} at {} line {} column {}",
                msg.unwrap(),
                loc.file,
                loc.line,
                loc.column,
            )
        })
    }
}
