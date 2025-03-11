use std::collections::HashMap;

use crate::{adapter::Symbol, value::Value};

pub(super) type Environment = HashMap<Symbol, Value>;
