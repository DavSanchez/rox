use std::{
    alloc::{self, Layout},
    fmt,
    ptr::{self, NonNull},
};

#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Return => write!(f, "OP_RETURN"),
        }
    }
}

pub type ByteCodeChunk = Chunk<OpCode>;

impl ByteCodeChunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        for i in 0..self.length() {
            let opcode = unsafe { ptr::read(self.ptr.as_ptr().add(i)) };
            println!("{i:04} {opcode}");
        }
    }
}

#[derive(Debug)]
pub struct Chunk<T> {
    length: usize,
    capacity: usize,
    ptr: NonNull<T>,
}

impl<T> Chunk<T> {
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

impl<T> Drop for Chunk<T> {
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
