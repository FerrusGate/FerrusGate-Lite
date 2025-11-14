pub mod args;
mod r#impl;
mod structs;

pub use r#impl::{get_config, get_config_path, init_config};
pub use structs::*;
