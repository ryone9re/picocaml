use std::collections::HashMap;

use crate::adapter::Symbol;

pub(super) type TypeEnvironment = HashMap<Symbol, ()>;
