//! 3 Pass parser for Google Standard SQL (Spanner dialect, no support for BigQuery specific
//! features).
//!
//! # Passes:
//!
//! # 1: Tokenize text
//! # 2: Assemble AST
//! # 3: Evaluate (TODO)
#![feature(pattern, array_try_from_fn, result_flattening)]

pub mod ast;
pub mod error;
// mod map;
pub mod tokens;
pub mod types;

pub use error::Error;
