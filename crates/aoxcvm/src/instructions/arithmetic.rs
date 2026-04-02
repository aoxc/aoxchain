//! Deterministic arithmetic semantics used by the phase-3 execution prototype.

/// Canonical arithmetic traps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticTrap {
    StackUnderflow,
    Overflow,
    DivisionByZero,
}

#[inline]
pub fn add(lhs: i64, rhs: i64) -> Result<i64, ArithmeticTrap> {
    lhs.checked_add(rhs).ok_or(ArithmeticTrap::Overflow)
}

#[inline]
pub fn sub(lhs: i64, rhs: i64) -> Result<i64, ArithmeticTrap> {
    lhs.checked_sub(rhs).ok_or(ArithmeticTrap::Overflow)
}

#[inline]
pub fn mul(lhs: i64, rhs: i64) -> Result<i64, ArithmeticTrap> {
    lhs.checked_mul(rhs).ok_or(ArithmeticTrap::Overflow)
}

#[inline]
pub fn div(lhs: i64, rhs: i64) -> Result<i64, ArithmeticTrap> {
    if rhs == 0 {
        return Err(ArithmeticTrap::DivisionByZero);
    }
    lhs.checked_div(rhs).ok_or(ArithmeticTrap::Overflow)
}

#[inline]
pub fn rem(lhs: i64, rhs: i64) -> Result<i64, ArithmeticTrap> {
    if rhs == 0 {
        return Err(ArithmeticTrap::DivisionByZero);
    }
    lhs.checked_rem(rhs).ok_or(ArithmeticTrap::Overflow)
}

#[cfg(test)]
mod tests {
    use super::{ArithmeticTrap, add, div, mul, rem, sub};

    #[test]
    fn checked_overflow_traps() {
        assert_eq!(add(i64::MAX, 1), Err(ArithmeticTrap::Overflow));
        assert_eq!(sub(i64::MIN, 1), Err(ArithmeticTrap::Overflow));
        assert_eq!(mul(i64::MAX, 2), Err(ArithmeticTrap::Overflow));
    }

    #[test]
    fn divide_by_zero_traps() {
        assert_eq!(div(10, 0), Err(ArithmeticTrap::DivisionByZero));
        assert_eq!(rem(10, 0), Err(ArithmeticTrap::DivisionByZero));
    }

    #[test]
    fn deterministic_results() {
        assert_eq!(add(7, 2), Ok(9));
        assert_eq!(sub(7, 2), Ok(5));
        assert_eq!(mul(7, 2), Ok(14));
        assert_eq!(div(7, 2), Ok(3));
        assert_eq!(rem(7, 2), Ok(1));
    }
}
