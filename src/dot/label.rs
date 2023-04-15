use std::fmt::{Display, Formatter, Result};

/// A trait that provides a label for a node in graph visualization.
pub trait Label {
	/// # Errors
	///
	/// If writing to the formatter fails.
	fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}

/// A reference to a type that forwards its [`Display`] to [`Label::fmt`].
pub struct Ref<'a, T>(pub &'a T);

impl<'a, T: Label> Display for Ref<'a, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		Label::fmt(self.0, f)
	}
}
