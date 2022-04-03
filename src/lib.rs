pub mod error;
pub mod parser;
#[cfg(any(feature = "brainfxck", feature = "ook"))]
pub mod predefined;
pub mod program;
pub mod runtime;
pub mod token;

pub mod prelude {
    pub use crate::error::*;
    pub use crate::parser::*;
    #[cfg(feature = "brainfxck")]
    pub use crate::predefined::brainfxck::*;
    #[cfg(feature = "ook")]
    pub use crate::predefined::ook::*;
    #[cfg(any(feature = "brainfxck", feature = "ook"))]
    pub use crate::predefined::*;
    pub use crate::program::*;
    pub use crate::runtime::*;
    pub use crate::token::simple::*;
    pub use crate::token::*;
}
