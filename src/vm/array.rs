use std::{
    alloc::{self, Layout},
    ops::Index,
    ptr::{self, NonNull},
};

#[derive(Debug)]
pub struct Array<T> {
    length: usize,
    capacity: usize,
    ptr: NonNull<T>,
}

impl<T> Array<T> {
    pub(super) fn new() -> Self {
        Self {
            length: 0,
            capacity: 0,
            ptr: NonNull::dangling(),
        }
    }

    pub(super) fn push(&mut self, byte: T) {
        if self.length == self.capacity {
            self.grow();
        }

        unsafe { ptr::write(self.ptr.as_ptr().add(self.length), byte) }

        // Can't fail, we'll OOM first.
        self.length += 1;
    }

    fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.length))) }
        }
    }

    pub(super) fn count(&self) -> usize {
        self.length
    }

    fn grow(&mut self) {
        // This makes use of `Layout::array` which creates a memory layout matching a `[T; n]`.
        let (new_capacity, new_layout) = if self.capacity == 0 {
            (1, Layout::array::<T>(1))
        } else {
            let new_capacity = self.capacity * 2;
            (new_capacity, Layout::array::<T>(new_capacity))
        };

        // This `Layout` is used on `alloc` and `realloc`.
        let new_layout = new_layout.expect("Allocation too large");

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.capacity)
                .expect("Previous allocation would have been too large");
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.capacity = new_capacity;
    }
}

impl<T> Index<usize> for Array<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.length, "Index out of bounds");
        // We do not use `ptr::read` here because we want to return a reference.
        unsafe { &*self.ptr.as_ptr().add(index) }
    }
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            while self.pop().is_some() { /* pop all elements */ }
            let layout = Layout::array::<T>(self.capacity)
                .expect("Previous allocation would have been too large");
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_push_and_index() {
        let mut array = Array::new();
        array.push(10);
        array.push(20);

        assert_eq!(array.count(), 2);
        assert_eq!(array[0], 10);
        assert_eq!(array[1], 20);
    }

    #[test]
    fn test_pop() {
        let mut array = Array::new();
        array.push(10);
        array.push(20);

        assert_eq!(array.pop(), Some(20));
        assert_eq!(array.count(), 1);
        assert_eq!(array.pop(), Some(10));
        assert_eq!(array.count(), 0);
        assert_eq!(array.pop(), None);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_out_of_bounds() {
        let mut array: Array<u8> = Array::new();
        array.push(1);
        let _ = array[1];
    }

    proptest! {
        #[test]
        fn prop_push_and_read(vals in proptest::collection::vec(any::<u8>(), 0..100)) {
            let mut array = Array::new();
            for val in &vals {
                array.push(*val);
            }

            prop_assert_eq!(array.count(), vals.len());

            for (i, val) in vals.iter().enumerate() {
                prop_assert_eq!(&array[i], val);
            }
        }

        #[test]
        fn prop_push_pop(vals in proptest::collection::vec(any::<u8>(), 0..100)) {
            let mut array = Array::new();
            for val in &vals {
                array.push(*val);
            }

            let mut reversed = vals.clone();
            reversed.reverse();

            for val in reversed {
                prop_assert_eq!(array.pop(), Some(val));
            }
            prop_assert_eq!(array.pop(), None);
            prop_assert_eq!(array.count(), 0);
        }
    }
}
