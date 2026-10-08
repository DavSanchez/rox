use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

use super::object::Obj;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Rc<Obj>),
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::Obj(obj) if obj.as_string().is_some())
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::Obj(obj) => obj.as_string(),
            _ => None,
        }
    }

    pub fn is_falsey(&self) -> bool {
        matches!(self, Self::Nil | Self::Bool(false))
    }

    pub fn values_equal(&self, other: &Self) -> bool {
        // `PartialEq` already implements the book's `valuesEqual`: same variant,
        // and equal payload per that type's own `PartialEq` (so `Number`s compare
        // like IEEE 754 `f64`, e.g. NaN != NaN, and `Obj`s compare by contents).
        self == other
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => value.fmt(f),
            Self::Nil => f.write_str("nil"),
            Self::Number(value) => value.fmt(f),
            Self::Obj(obj) => obj.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::object::ObjString;

    fn string(chars: &str) -> Value {
        Value::Obj(Obj::from_string(ObjString::copy(chars)))
    }

    #[test]
    fn displays_each_type() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
        assert_eq!(Value::Nil.to_string(), "nil");
        assert_eq!(Value::Number(1.5).to_string(), "1.5");
        assert_eq!(string("lox").to_string(), "lox");
    }

    #[test]
    fn only_nil_and_false_are_falsey() {
        assert!(Value::Nil.is_falsey());
        assert!(Value::Bool(false).is_falsey());
        assert!(!Value::Bool(true).is_falsey());
        assert!(!Value::Number(0.0).is_falsey());
        assert!(!string("").is_falsey());
    }

    #[test]
    fn values_compare_by_type_and_value() {
        assert!(Value::Nil.values_equal(&Value::Nil));
        assert!(Value::Bool(true).values_equal(&Value::Bool(true)));
        assert!(Value::Number(1.0).values_equal(&Value::Number(1.0)));
        assert!(!Value::Bool(true).values_equal(&Value::Number(1.0)));
        assert!(!Value::Nil.values_equal(&Value::Bool(false)));
    }

    #[test]
    fn strings_compare_by_contents() {
        let left = string("lox");
        let right = string("lox");
        let different = string("clox");

        assert!(left.values_equal(&right));
        assert!(!left.values_equal(&different));
        assert!(!left.values_equal(&Value::Number(1.0)));
    }

    #[test]
    fn exposes_underlying_values() {
        assert_eq!(Value::Number(1.5).as_number(), Some(1.5));
        assert_eq!(Value::Nil.as_number(), None);
        assert!(string("lox").is_string());
        assert_eq!(string("lox").as_string(), Some("lox"));
        assert_eq!(Value::Number(1.0).as_string(), None);
    }
}
