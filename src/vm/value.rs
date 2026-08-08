use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl Value {
    pub fn as_number(self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_falsey(self) -> bool {
        matches!(self, Self::Nil | Self::Bool(false))
    }

    pub fn values_equal(self, other: Self) -> bool {
        match (self, other) {
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Nil, Self::Nil) => true,
            (Self::Number(left), Self::Number(right)) => left == right,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => value.fmt(f),
            Self::Nil => f.write_str("nil"),
            Self::Number(value) => value.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Value;

    #[test]
    fn displays_each_type() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
        assert_eq!(Value::Nil.to_string(), "nil");
        assert_eq!(Value::Number(1.5).to_string(), "1.5");
    }

    #[test]
    fn only_nil_and_false_are_falsey() {
        assert!(Value::Nil.is_falsey());
        assert!(Value::Bool(false).is_falsey());
        assert!(!Value::Bool(true).is_falsey());
        assert!(!Value::Number(0.0).is_falsey());
    }

    #[test]
    fn values_compare_by_type_and_value() {
        assert!(Value::Nil.values_equal(Value::Nil));
        assert!(Value::Bool(true).values_equal(Value::Bool(true)));
        assert!(Value::Number(1.0).values_equal(Value::Number(1.0)));
        assert!(!Value::Bool(true).values_equal(Value::Number(1.0)));
        assert!(!Value::Nil.values_equal(Value::Bool(false)));
    }
}
