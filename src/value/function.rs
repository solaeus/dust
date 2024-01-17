use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{BuiltInFunction, Format, FunctionNode, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Function {
    BuiltIn(BuiltInFunction),
    ContextDefined(FunctionNode),
}

impl Function {
    pub fn call(&self, arguments: &[Value], source: &str, outer_context: &Map) -> Result<Value> {
        match self {
            Function::BuiltIn(built_in_function) => {
                built_in_function.call(arguments, source, outer_context)
            }
            Function::ContextDefined(context_defined_function) => {
                context_defined_function.call(arguments, source, outer_context)
            }
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Function::BuiltIn(built_in_function) => built_in_function.r#type(),
            Function::ContextDefined(context_defined_function) => {
                context_defined_function.r#type().clone()
            }
        }
    }
}

impl Format for Function {
    fn format(&self, output: &mut String, indent_level: u8) {
        match self {
            Function::BuiltIn(built_in_function) => built_in_function.format(output, indent_level),
            Function::ContextDefined(function_node) => function_node.format(output, indent_level),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Function::BuiltIn(built_in_function) => write!(f, "{built_in_function}"),
            Function::ContextDefined(function_node) => write!(f, "{function_node}"),
        }
    }
}
