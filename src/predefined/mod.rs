//! Predefined Brainfuck-like implementations.
//!
//! This module is enabled when predefined related features are enabled.
#[cfg(feature = "bf")]
#[cfg_attr(docsrs, doc(cfg(feature = "bf")))]
pub mod bf;

#[cfg(feature = "ook")]
#[cfg_attr(docsrs, doc(cfg(feature = "ook")))]
pub mod ook;
