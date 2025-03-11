use std::collections::HashMap;

use crate::adapter::Symbol;

pub type TypeEnvironment = HashMap<Symbol, ()>;
