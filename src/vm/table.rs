use std::rc::Rc;

use super::object::{ObjString, hash_string};
use super::value::Value;
use crate::array::Array;

/// Mirrors the book's `TABLE_MAX_LOAD`. The table grows once it is this full.
const TABLE_MAX_LOAD: f64 = 0.75;

/// Mirrors the book's `GROW_CAPACITY` for tables: start at eight buckets, then
/// double.
fn grow_capacity(capacity: usize) -> usize {
    if capacity < 8 { 8 } else { capacity * 2 }
}

/// A single bucket in a [`Table`].
///
/// The book tells empty buckets and tombstones apart using a `NULL` key plus a
/// `nil`/`true` value. An enum makes that distinction explicit. Keys are always
/// strings, so the entry stores an [`ObjString`] directly, like the book's
/// `ObjString* key`.
#[allow(dead_code)]
#[derive(Debug)]
enum Entry {
    Empty,
    Tombstone,
    Occupied(Rc<ObjString>, Value),
}

/// A string-keyed hash table, mirroring the book's `Table`.
///
/// Entries are kept in a contiguous bucket array and collisions are resolved
/// with linear probing. `count` includes tombstones, just like the book, so the
/// load factor never hides an array that has no truly empty bucket left.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Table {
    count: usize,
    capacity: usize,
    entries: Array<Entry>,
}

#[allow(dead_code)]
impl Table {
    /// The number of occupied buckets plus tombstones.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Adds or overwrites the entry for `key`, mirroring the book's
    /// `tableSet()`. Returns `true` when a new key was added.
    pub fn set(&mut self, key: Rc<ObjString>, value: Value) -> bool {
        if (self.count + 1) as f64 > self.capacity as f64 * TABLE_MAX_LOAD {
            let capacity = grow_capacity(self.capacity);
            self.adjust_capacity(capacity);
        }

        let index = find_entry(&self.entries, self.capacity, &key);
        let is_new_key = matches!(&self.entries[index], Entry::Empty);
        if is_new_key {
            self.count += 1;
        }
        self.entries[index] = Entry::Occupied(key, value);
        is_new_key
    }

    /// Looks up the value for `key`, mirroring the book's `tableGet()`.
    pub fn get(&self, key: &Rc<ObjString>) -> Option<&Value> {
        if self.count == 0 {
            return None;
        }

        let index = find_entry(&self.entries, self.capacity, key);
        match &self.entries[index] {
            Entry::Occupied(_, value) => Some(value),
            _ => None,
        }
    }

    /// Removes the entry for `key` by leaving a tombstone, mirroring the book's
    /// `tableDelete()`. Returns `true` when an entry was removed.
    pub fn delete(&mut self, key: &Rc<ObjString>) -> bool {
        if self.count == 0 {
            return false;
        }

        let index = find_entry(&self.entries, self.capacity, key);
        if matches!(&self.entries[index], Entry::Occupied(..)) {
            self.entries[index] = Entry::Tombstone;
            true
        } else {
            false
        }
    }

    /// Copies every entry of `from` into this table, mirroring the book's
    /// `tableAddAll()`.
    pub fn add_all(&mut self, from: &Self) {
        for entry in from.entries.iter() {
            if let Entry::Occupied(key, value) = entry {
                self.set(key.clone(), value.clone());
            }
        }
    }

    /// Finds an existing string in the table, or returns `None`.
    ///
    /// Mirrors the book's `tableFindString()`. Unlike [`Table::set`] and
    /// [`Table::get`], this compares characters so a string can be deduplicated
    /// *before* an [`ObjString`] is allocated for it.
    pub fn find_string(&self, chars: &str, hash: u32) -> Option<Rc<ObjString>> {
        if self.count == 0 {
            return None;
        }

        let mut index = hash as usize % self.capacity;
        loop {
            match &self.entries[index] {
                Entry::Empty => return None,
                Entry::Occupied(key, _) => {
                    if key.hash() == hash && key.as_str() == chars {
                        return Some(key.clone());
                    }
                }
                Entry::Tombstone => {}
            }
            index = (index + 1) % self.capacity;
        }
    }

    /// Interns the given characters, reusing an existing string if present.
    ///
    /// Mirrors the book's `copyString()`.
    pub fn intern(&mut self, chars: &str) -> Rc<ObjString> {
        let hash = hash_string(chars);
        if let Some(interned) = self.find_string(chars, hash) {
            return interned;
        }

        let string = ObjString::copy(chars);
        self.set(string.clone(), Value::Nil);
        string
    }

    /// Interns characters owned by the caller, reusing an existing string if
    /// present. Mirrors the book's `takeString()`.
    pub fn intern_take(&mut self, chars: String) -> Rc<ObjString> {
        let hash = hash_string(&chars);
        if let Some(interned) = self.find_string(&chars, hash) {
            return interned;
        }

        let string = ObjString::take(chars);
        self.set(string.clone(), Value::Nil);
        string
    }

    /// Rebuilds the bucket array at `capacity`, mirroring the book's
    /// `adjustCapacity()`. Because a key's bucket depends on the array size,
    /// every entry is reinserted and tombstones are discarded.
    fn adjust_capacity(&mut self, capacity: usize) {
        let mut entries = Array::default();
        for _ in 0..capacity {
            entries.push(Entry::Empty);
        }

        self.count = 0;
        let old_entries = std::mem::take(&mut self.entries);
        for entry in old_entries.iter() {
            if let Entry::Occupied(key, value) = entry {
                let index = find_entry(&entries, capacity, key);
                entries[index] = Entry::Occupied(key.clone(), value.clone());
                self.count += 1;
            }
        }

        self.entries = entries;
        self.capacity = capacity;
    }
}

/// Returns the bucket `key` belongs in, mirroring the book's `findEntry()`.
///
/// Collisions are resolved by linear probing. The returned slot is either empty,
/// a reusable tombstone, or the entry that already holds `key`.
fn find_entry(entries: &[Entry], capacity: usize, key: &Rc<ObjString>) -> usize {
    let mut index = key.hash() as usize % capacity;
    let mut tombstone: Option<usize> = None;

    loop {
        match &entries[index] {
            Entry::Empty => return tombstone.unwrap_or(index),
            Entry::Tombstone => {
                if tombstone.is_none() {
                    tombstone = Some(index);
                }
            }
            Entry::Occupied(existing, _) => {
                if existing == key {
                    return index;
                }
            }
        }
        index = (index + 1) % capacity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Finds two distinct keys whose hashes land in the same bucket.
    fn find_collision(capacity: usize) -> (Rc<ObjString>, Rc<ObjString>) {
        let mut seen: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
        for i in 0.. {
            let chars = format!("key{i}");
            let bucket = hash_string(&chars) as usize % capacity;
            if let Some(previous) = seen.get(&bucket) {
                return (ObjString::copy(previous), ObjString::copy(&chars));
            }
            seen.insert(bucket, chars);
        }
        unreachable!()
    }

    #[test]
    fn set_stores_and_overwrites() {
        let key = ObjString::copy("lox");
        let mut table = Table::default();

        assert!(table.set(key.clone(), Value::Number(1.0)));
        assert_eq!(table.count(), 1);
        assert_eq!(table.get(&key).and_then(Value::as_number), Some(1.0));

        // Overwriting an existing key does not add an entry.
        assert!(!table.set(key.clone(), Value::Number(2.0)));
        assert_eq!(table.count(), 1);
        assert_eq!(table.get(&key).and_then(Value::as_number), Some(2.0));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let mut table = Table::default();
        assert!(table.get(&ObjString::copy("missing")).is_none());

        table.set(ObjString::copy("lox"), Value::Number(1.0));
        assert!(table.get(&ObjString::copy("missing")).is_none());
    }

    #[test]
    fn resize_preserves_every_entry() {
        let mut table = Table::default();
        let keys: Vec<Rc<ObjString>> = (0..100)
            .map(|i| ObjString::copy(&format!("key{i}")))
            .collect();

        for (i, key) in keys.iter().enumerate() {
            table.set(key.clone(), Value::Number(i as f64));
        }

        assert!(table.capacity > 8);
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(table.get(key).and_then(Value::as_number), Some(i as f64));
        }
    }

    #[test]
    fn delete_missing_key_returns_false() {
        let mut table = Table::default();
        assert!(!table.delete(&ObjString::copy("missing")));
    }

    #[test]
    fn tombstones_keep_probe_chains_intact() {
        let (first, second) = find_collision(8);
        let mut table = Table::default();

        table.set(first.clone(), Value::Number(1.0));
        table.set(second.clone(), Value::Number(2.0));

        assert!(table.delete(&first));
        assert!(table.get(&first).is_none());
        // `second` must still be reachable even though it sits past a tombstone.
        assert_eq!(table.get(&second).and_then(Value::as_number), Some(2.0));
    }

    #[test]
    fn tombstones_are_reused_on_insert() {
        let (first, second) = find_collision(8);
        let mut table = Table::default();

        table.set(first.clone(), Value::Number(1.0));
        table.set(second.clone(), Value::Number(2.0));
        assert!(table.delete(&first));
        let count_after_delete = table.count();

        // The tombstone left by `first` should be reused, not counted again.
        let replacement = ObjString::copy("replacement");
        table.set(replacement.clone(), Value::Number(3.0));
        assert!(table.get(&replacement).is_some());
        assert_eq!(table.count(), count_after_delete + 1);
    }

    #[test]
    fn find_string_deduplicates() {
        let mut interner = Table::default();
        let original = interner.intern("lox");

        let found = interner
            .find_string("lox", hash_string("lox"))
            .expect("interned string should be found");
        assert!(Rc::ptr_eq(&original, &found));
        assert!(interner.find_string("clox", hash_string("clox")).is_none());
    }

    #[test]
    fn intern_reuses_the_same_object() {
        let mut interner = Table::default();

        let copied = interner.intern("lox");
        let again = interner.intern("lox");
        let taken = interner.intern_take(String::from("lox"));
        let different = interner.intern("clox");

        assert!(Rc::ptr_eq(&copied, &again));
        assert!(Rc::ptr_eq(&copied, &taken));
        assert!(!Rc::ptr_eq(&copied, &different));
    }

    #[test]
    fn add_all_copies_entries() {
        let key = ObjString::copy("lox");
        let mut from = Table::default();
        from.set(key.clone(), Value::Number(1.0));

        let mut to = Table::default();
        to.add_all(&from);

        assert_eq!(to.get(&key).and_then(Value::as_number), Some(1.0));
    }
}
