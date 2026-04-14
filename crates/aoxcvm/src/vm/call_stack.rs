//! Deterministic bounded call stack.

use crate::vm::frame::Frame;

/// Bounded call stack used by the VM for intra-program calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallStack {
    frames: Vec<Frame>,
    max_depth: usize,
}

impl CallStack {
    /// Creates a call stack with a deterministic depth bound.
    pub fn new(max_depth: usize) -> Self {
        Self {
            frames: Vec::with_capacity(max_depth.min(64)),
            max_depth,
        }
    }

    /// Pushes a frame, returning `false` when max depth is exceeded.
    pub fn push(&mut self, frame: Frame) -> bool {
        if self.frames.len() >= self.max_depth {
            return false;
        }

        self.frames.push(frame);
        true
    }

    /// Pops the top-most frame.
    pub fn pop(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    /// Borrows the top-most frame.
    pub fn peek(&self) -> Option<&Frame> {
        self.frames.last()
    }

    /// Returns true when no active frames are present.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Returns the current frame count.
    pub fn len(&self) -> usize {
        self.frames.len()
    }
}

#[cfg(test)]
mod tests {
    use super::CallStack;
    use crate::vm::frame::Frame;

    #[test]
    fn call_stack_respects_depth_limit() {
        let mut stack = CallStack::new(1);
        assert!(stack.push(Frame::new(4, 0)));
        assert!(!stack.push(Frame::new(7, 1)));
        assert_eq!(stack.pop(), Some(Frame::new(4, 0)));
        assert!(stack.is_empty());
    }
}
