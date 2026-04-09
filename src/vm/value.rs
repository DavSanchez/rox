use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Value(f64);

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
