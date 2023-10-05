//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree as TSTree, TreeCursor};

use crate::{language, Error, Result, Value, VariableMap};
