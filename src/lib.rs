#![feature(type_alias_impl_trait)]

// tools lib
pub mod object;
pub mod service;
pub mod utils;

// export parse
pub use condition::parse;

#[macro_use]
pub mod store;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = anyhow::Result<T>;
