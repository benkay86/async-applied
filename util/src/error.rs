//! Custom types for error handling.

/// Type-erased error that can be moved between threads.  Example:
/// 
/// ```ignore
/// fn my_func() -> Result<(), BoxError> {
///     do_something_fallible()?;
///     Ok(())
/// }
/// ```
/// 
/// See https://github.com/rust-lang/rfcs/pull/2820
pub type BoxError = std::boxed::Box<dyn
    std::error::Error   // must implement Error to satisfy ?
	+ std::marker::Send // needed for threads
	+ std::marker::Sync // needed for threads
>;

/// Generic error type that stores a message `what` and optionally wraps another
/// causative error `source`.  Example:
/// 
/// ```ignore
/// fn my_func() -> Result<(), BoxError> {
///     match do_something_fallible() {
///         Ok(_) => (),
///         Err(e) => return Err(Error{
///             what: "There was a problem doing something.".to_string(),
///             source: Some(e),
///         }.into()),
///     };
///     
///     if did_it_work() == false {
///         return Err(Error{
///             what: "It didn't work.".to_string(),
///             source: None,
///         }.into());
///     }
///     
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Error {
    /// What went wrong?
	pub what: String,
	/// What was the source/cause of this error, if any?
	pub source: Option<BoxError>
}
impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.what)?;
		if let Some(error) = &self.source {
			write!(f, "\nCaused by: {}", error)?;
		}
		Ok(())
	}
}
impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match &self.source {
			Some(error) => Some(error.as_ref()),
			None => None
		}
	}
}
