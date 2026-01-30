mod array;
pub mod opcode;
pub mod value;

use std::ptr;

use array::Array;
use opcode::OpCode;
use value::Value;

pub struct Chunk {
    pub code_array: Array<OpCode>,
    pub _value_array: Array<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code_array: Array::new(),
            _value_array: Array::new(),
        }
    }
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        for i in 0..self.code_array.length() {
            let opcode = unsafe { ptr::read(self.code_array.ptr.as_ptr().add(i)) };
            println!("{i:04} {opcode}");
        }
    }
}
