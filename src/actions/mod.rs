pub mod core;
mod types;
pub use types::*;
mod command_parser;
pub mod context;

use anyhow::Result;

pub type ActionResult = Result<()>;