use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

/// Hashes the given characters with the FNV-1a algorithm, mirroring the book's
/// `hashString()`.
pub fn hash_string(key: &str) -> u32 {
    let mut hash: u32 = 2_166_136_261;
    for byte in key.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

/// A heap-allocated string.
///
/// Mirrors the book's `ObjString`: the characters plus a hash code cached the
/// moment the string is created. Because Lox strings are immutable, the hash
/// can never go stale.
#[derive(Debug, PartialEq)]
pub struct ObjString {
    chars: Rc<str>,
    hash: u32,
}

impl ObjString {
    fn new(chars: Rc<str>) -> Self {
        let hash = hash_string(&chars);
        Self { chars, hash }
    }

    pub fn as_str(&self) -> &str {
        &self.chars
    }

    pub fn hash(&self) -> u32 {
        self.hash
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.chars)
    }
}

/// A heap-allocated value.
///
/// Mirrors the book's `Obj`: every value whose state lives on the heap is one
/// of these. Strings are the only object type for now; later chapters add
/// functions, closures, classes, and instances.
#[derive(Debug, PartialEq)]
pub enum Obj {
    String(ObjString),
}

impl Obj {
    /// Copies the given characters onto the heap, mirroring the book's
    /// `copyString()`. Used for string literals, whose characters live inside
    /// the source string.
    pub fn copy_string(chars: &str) -> Rc<Self> {
        Rc::new(Self::String(ObjString::new(Rc::from(chars))))
    }

    /// Takes ownership of the given characters, mirroring the book's
    /// `takeString()`. Used for strings built at runtime, such as
    /// concatenations, so their characters are not copied again.
    pub fn take_string(chars: String) -> Rc<Self> {
        Rc::new(Self::String(ObjString::new(Rc::from(chars))))
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string.as_str()),
        }
    }

    /// The cached hash code of this object. Only strings have one for now.
    pub fn hash(&self) -> u32 {
        match self {
            Self::String(string) => string.hash(),
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(string) => string.fmt(f),
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

    #[test]
    fn hash_is_deterministic_and_content_based() {
        assert_eq!(hash_string("lox"), hash_string("lox"));
        assert_ne!(hash_string("lox"), hash_string("clox"));
        assert_eq!(hash_string(""), 2_166_136_261);
    }

    #[test]
    fn strings_cache_their_hash() {
        let object = Obj::copy_string("lox");
        assert_eq!(object.hash(), hash_string("lox"));

        // `take_string` computes the same hash for the same characters.
        let taken = Obj::take_string(String::from("lox"));
        assert_eq!(taken.hash(), hash_string("lox"));
    }

    #[test]
    fn hash_handles_non_ascii() {
        let chars = "A~¶Þॐஃ";
        let object = Obj::copy_string(chars);
        assert_eq!(object.hash(), hash_string(chars));
    }
}
