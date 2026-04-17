use std::fmt;

use super::value::Value;

const STACK_MAX: usize = 256;

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
            slots: [0.0.into(); STACK_MAX],
            top: 0,
        }
    }
}

impl ValueStack {
    pub(super) fn push(&mut self, value: Value) {
        debug_assert!(self.top < STACK_MAX, "stack overflow");
        self.slots[self.top] = value;
        self.top += 1;
    }

    pub(super) fn pop(&mut self) -> Value {
        debug_assert!(self.top > 0, "stack underflow");
        self.top -= 1;
        self.slots[self.top]
    }
}
