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
/// can never go stale. This is also the key type of every [`Table`], just like
/// the book's `ObjString*`.
///
/// [`Table`]: crate::vm::table::Table
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

    /// Copies the given characters onto the heap, mirroring the book's
    /// `copyString()`. Used for string literals, whose characters live inside
    /// the source string.
    pub fn copy(chars: &str) -> Rc<Self> {
        Rc::new(Self::new(Rc::from(chars)))
    }

    /// Takes ownership of the given characters, mirroring the book's
    /// `takeString()`. Used for strings built at runtime, such as
    /// concatenations, so their characters are not copied again.
    pub fn take(chars: String) -> Rc<Self> {
        Rc::new(Self::new(Rc::from(chars)))
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
    String(Rc<ObjString>),
}

impl Obj {
    /// Wraps an already-allocated string as a value object, like the implicit
    /// `Obj*` upcast in the book.
    pub fn from_string(string: Rc<ObjString>) -> Rc<Self> {
        Rc::new(Self::String(string))
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string.as_str()),
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(string) => f.write_str(string.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_string_owns_its_characters() {
        let source = String::from("hello");
        let object = ObjString::copy(&source);
        drop(source);

        assert_eq!(object.as_str(), "hello");
    }

    #[test]
    fn take_string_owns_its_characters() {
        let object = ObjString::take(String::from("world"));

        assert_eq!(object.as_str(), "world");
    }

    #[test]
    fn strings_compare_by_contents() {
        assert_eq!(ObjString::copy("lox"), ObjString::take(String::from("lox")));
        assert_ne!(ObjString::copy("lox"), ObjString::copy("clox"));
    }

    #[test]
    fn displays_characters_without_quotes() {
        let object = Obj::from_string(ObjString::copy("a string"));
        assert_eq!(object.to_string(), "a string");

        let empty = Obj::from_string(ObjString::copy(""));
        assert_eq!(empty.to_string(), "");
    }

    #[test]
    fn hash_is_deterministic_and_content_based() {
        assert_eq!(hash_string("lox"), hash_string("lox"));
        assert_ne!(hash_string("lox"), hash_string("clox"));
        assert_eq!(hash_string(""), 2_166_136_261);
    }

    #[test]
    fn strings_cache_their_hash() {
        let object = ObjString::copy("lox");
        assert_eq!(object.hash(), hash_string("lox"));

        // `take` computes the same hash for the same characters.
        let taken = ObjString::take(String::from("lox"));
        assert_eq!(taken.hash(), hash_string("lox"));
    }

    #[test]
    fn hash_handles_non_ascii() {
        let chars = "A~¶Þॐஃ";
        let object = ObjString::copy(chars);
        assert_eq!(object.hash(), hash_string(chars));
    }
}
