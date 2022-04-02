pub mod error;
pub mod parser;
pub mod predefined;
pub mod program;
pub mod runtime;
pub mod token;

pub mod prelude {
    pub use crate::error::*;
    pub use crate::parser::*;
    pub use crate::predefined::brainfxck::*;
    pub use crate::predefined::ook::*;
    pub use crate::predefined::*;
    pub use crate::program::*;
    pub use crate::runtime::*;
    pub use crate::token::simple::*;
    pub use crate::token::*;
}
