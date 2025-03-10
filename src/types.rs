use std::collections::HashMap;

pub type RInteger = isize;
pub type RBool = bool;

pub type Symbol = String;

pub type TypeEnvironment = HashMap<Symbol, ()>;
