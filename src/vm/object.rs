use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

/// A heap-allocated value.
///
/// Mirrors the book's `Obj`: every value whose state lives on the heap is one
/// of these. Strings are the only object type for now; later chapters add
/// functions, closures, classes, and instances.
#[derive(Debug, PartialEq)]
pub enum Obj {
    String(Rc<str>),
}

impl Obj {
    /// Copies the given characters onto the heap, mirroring the book's
    /// `copyString()`. Used for string literals, whose characters live inside
    /// the source string.
    pub fn copy_string(chars: &str) -> Rc<Self> {
        Rc::new(Self::String(Rc::from(chars)))
    }

    /// Takes ownership of the given characters, mirroring the book's
    /// `takeString()`. Used for strings built at runtime, such as
    /// concatenations, so their characters are not copied again.
    pub fn take_string(chars: String) -> Rc<Self> {
        Rc::new(Self::String(Rc::from(chars)))
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(chars) => Some(chars),
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(chars) => f.write_str(chars),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_string_owns_its_characters() {
        let source = String::from("hello");
        let object = Obj::copy_string(&source);
        drop(source);

        assert_eq!(object.as_string(), Some("hello"));
    }

    #[test]
    fn take_string_owns_its_characters() {
        let object = Obj::take_string(String::from("world"));

        assert_eq!(object.as_string(), Some("world"));
    }

    #[test]
    fn strings_compare_by_contents() {
        assert_eq!(
            Obj::copy_string("lox"),
            Obj::take_string(String::from("lox"))
        );
        assert_ne!(Obj::copy_string("lox"), Obj::copy_string("clox"));
    }

    #[test]
    fn displays_characters_without_quotes() {
        assert_eq!(Obj::copy_string("a string").to_string(), "a string");
        assert_eq!(Obj::copy_string("").to_string(), "");
    }
}
