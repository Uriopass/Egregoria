use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct MultiError<T>(pub Vec<T>);

impl<T: Display> Display for MultiError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for e in &self.0 {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}

impl<T: Error> Error for MultiError<T> {}
