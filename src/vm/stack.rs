use std::fmt;

use super::value::Value;

const STACK_MAX: usize = 256;

#[allow(dead_code)]
#[derive(Debug)]
pub(super) struct ValueStack {
    slots: [Value; STACK_MAX],
    top: usize,
}

impl fmt::Display for ValueStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        self.slots[..self.top]
            .iter()
            .try_for_each(|v| write!(f, " {v}"))?;
        write!(f, " ]")
    }
}

impl Default for ValueStack {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| Value::Nil),
            top: 0,
        }
    }
}

#[allow(dead_code)]
impl ValueStack {
    pub(super) fn push(&mut self, value: Value) {
        debug_assert!(self.top < STACK_MAX, "stack overflow");
        self.slots[self.top] = value;
        self.top += 1;
    }

    pub(super) fn pop(&mut self) -> Value {
        debug_assert!(self.top > 0, "stack underflow");
        self.top -= 1;
        std::mem::replace(&mut self.slots[self.top], Value::Nil)
    }

    pub(super) fn peek(&self, distance: usize) -> Value {
        debug_assert!(distance < self.top, "stack access out of bounds");
        self.slots[self.top - 1 - distance].clone()
    }

    pub(super) fn reset(&mut self) {
        self.top = 0;
    }
}
