use std::{
    alloc::{self, Layout},
    ptr::{self, NonNull},
};

#[derive(Debug)]
pub struct Array<T> {
    length: usize,
    capacity: usize,
    pub(super) ptr: NonNull<T>, // TODO remove pub(super) once we don't want to "disassemble"
}

impl<T> Array<T> {
    pub fn new() -> Self {
        Self {
            length: 0,
            capacity: 0,
            ptr: NonNull::dangling(),
        }
    }

    pub fn push(&mut self, byte: T) {
        if self.length == self.capacity {
            self.grow();
        }

        unsafe { ptr::write(self.ptr.as_ptr().add(self.length), byte) }

        // Can't fail, we'll OOM first.
        self.length += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.length))) }
        }
    }

    pub fn length(&self) -> usize {
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
