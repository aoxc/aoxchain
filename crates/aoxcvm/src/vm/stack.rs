//! Deterministic value-stack helpers.

/// VM stack with explicit maximum depth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueStack {
    values: Vec<i64>,
    max_depth: usize,
}

impl ValueStack {
    /// Creates a bounded stack.
    pub fn new(max_depth: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_depth.min(64)),
            max_depth,
        }
    }

    /// Pushes a value, returning `false` on overflow.
    pub fn push(&mut self, value: i64) -> bool {
        if self.values.len() >= self.max_depth {
            return false;
        }

        self.values.push(value);
        true
    }

    /// Pops the top value.
    pub fn pop(&mut self) -> Option<i64> {
        self.values.pop()
    }

    /// Returns the current top value by copy.
    pub fn peek(&self) -> Option<i64> {
        self.values.last().copied()
    }

    /// Returns number of stack elements.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Shrinks stack to a specific depth. No-op when `depth >= len`.
    pub fn truncate(&mut self, depth: usize) {
        self.values.truncate(depth);
    }
}

#[cfg(test)]
mod tests {
    use super::ValueStack;

    #[test]
    fn value_stack_is_bounded() {
        let mut stack = ValueStack::new(2);
        assert!(stack.push(10));
        assert!(stack.push(20));
        assert!(!stack.push(30));
        assert_eq!(stack.peek(), Some(20));
        assert_eq!(stack.pop(), Some(20));
        assert_eq!(stack.pop(), Some(10));
        assert_eq!(stack.pop(), None);
    }
}
