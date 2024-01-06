use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, BuiltInFunction, FunctionNode, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Function {
    BuiltIn(BuiltInFunction),
    ContextDefined(FunctionNode),
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Function::BuiltIn(built_in_function) => write!(f, "{}", built_in_function.r#type()),
            Function::ContextDefined(context_defined_function) => {
                write!(f, "{}", context_defined_function.r#type())
            }
        }
    }
}

impl Function {
    pub fn call(
        &self,
        name: Option<String>,
        arguments: &[Value],
        source: &str,
        outer_context: &Map,
    ) -> Result<Value> {
        match self {
            Function::BuiltIn(built_in_function) => {
                built_in_function.call(arguments, source, outer_context)
            }
            Function::ContextDefined(context_defined_function) => {
                context_defined_function.call(name, arguments, source, outer_context)
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

impl AbstractTree for Function {
    fn from_syntax_node(_source: &str, _node: Node, _context: &Map) -> Result<Self> {
        let inner_function = FunctionNode::from_syntax_node(_source, _node, _context)?;

        Ok(Function::ContextDefined(inner_function))
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<()> {
        match self {
            Function::BuiltIn(_) => Ok(()),
            Function::ContextDefined(defined_function) => {
                defined_function.check_type(_source, _context)
            }
        }
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::Function(self.clone()))
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        match self {
            Function::BuiltIn(built_in) => Ok(built_in.r#type()),
            Function::ContextDefined(context_defined) => Ok(context_defined.r#type().clone()),
        }
    }
}
