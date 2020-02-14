//! Crate containing common utilities shared by the other examples.

mod error;
/// Type-erased error that can be moved between threads.
pub use error::BoxError;
/// Generic error type that stores a message and can wrap other errors.
pub use error::Error;
