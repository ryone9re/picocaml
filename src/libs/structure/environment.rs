use std::collections::HashMap;

use crate::{adapter::Symbol, value::Value};

pub type Environment = HashMap<Symbol, Value>;
