//! The Dust programming language.
//!
//! To get started, you can use the `run` function to run a Dust program.
//!
//! ```rust
//! use dust_lang::{run, Value};
//!
//! let program = "
//!     let foo = 21
//!     let bar = 2
//!     foo * bar
//! ";
//!
//! let the_answer = run(program).unwrap();
//!
//! assert_eq!(the_answer, Some(Value::integer(42)));
//! ```
pub mod bytecode;
pub mod constructor;
pub mod identifier;
pub mod r#type;
pub mod value;

pub use bytecode::*;
pub use constructor::*;
pub use identifier::*;
pub use r#type::*;
pub use value::*;

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Span(usize, usize);

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
