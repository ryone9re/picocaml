use std::ops::{Add, Mul, Sub};

pub(crate) type RInteger = isize;
pub(crate) type RBool = bool;

pub(crate) type Symbol = String;

pub(crate) type RArithmeticOperation = fn(RInteger, RInteger) -> RInteger;

pub(crate) fn r_plus(lhs: RInteger, rhs: RInteger) -> RInteger {
    lhs.add(rhs)
}

pub(crate) fn r_minus(lhs: RInteger, rhs: RInteger) -> RInteger {
    lhs.sub(rhs)
}

pub(crate) fn r_times(lhs: RInteger, rhs: RInteger) -> RInteger {
    lhs.mul(rhs)
}

pub(crate) type RComparisonOperation = fn(RInteger, RInteger) -> RBool;

pub(crate) fn r_lt(lhs: RInteger, rhs: RInteger) -> RBool {
    lhs.lt(&rhs)
}
