//! Cancel an in-flight CFS session and clean up its derived resources.

pub mod command;
#[cfg(test)]
mod tests;

#[doc(inline)]
pub use command::exec;
