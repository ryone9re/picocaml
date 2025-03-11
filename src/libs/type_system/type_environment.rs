use std::collections::HashMap;

use crate::adapter::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Integer, // 基底型
    Bool,    // 基底型
    Variable { name: Symbol },
    Function { domain: Box<Type>, range: Box<Type> },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TypeEnvironment {
    equation: HashMap<Type, Type>, // 基底型と関数型の組はありえない
}
