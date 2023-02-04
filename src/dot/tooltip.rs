use std::fmt::{Display, Formatter, Result};

/// A trait that provides a tooltip for a node in graph visualization.
pub trait Tooltip {
	/// # Errors
	///
	/// If writing to the formatter fails.
	fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}

/// A reference to a type that forwards its [`Display`] to [`Tooltip::fmt`].
pub struct Ref<'a, T>(pub &'a T);

impl<'a, T> Display for Ref<'a, T>
where
	T: Tooltip,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		Tooltip::fmt(self.0, f)
	}
}
