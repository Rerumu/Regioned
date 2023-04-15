use std::fmt::{Display, Formatter, Result};

#[cfg(debug_assertions)]
pub type Id = arena::key::Id<std::num::NonZeroU32>;

#[cfg(not(debug_assertions))]
pub type Id = arena::key::Id<arena::version::Nil>;

/// A region.
///
/// It has a start and an end marker that delimit it.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region {
	start: Id,
	end: Id,
}

impl Region {
	pub(crate) const fn new(start: Id, end: Id) -> Self {
		Self { start, end }
	}

	/// Returns the start marker [`Id`] of the region.
	#[inline]
	#[must_use]
	pub const fn start(self) -> Id {
		self.start
	}

	/// Returns the end marker [`Id`] of the region.
	#[inline]
	#[must_use]
	pub const fn end(self) -> Id {
		self.end
	}
}

/// A marker node.
///
/// It is used to mark the start and end of a region.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Marker {
	/// The "start" node.
	///
	/// It represents the start and arguments to a region.
	Start,

	/// The "end" node.
	///
	/// It represents the end and returns of a region.
	End,
}

impl Display for Marker {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Self::Start => "Start".fmt(f),
			Self::End => "End".fmt(f),
		}
	}
}

/// A compound node.
///
/// It is used to represent a region.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Compound {
	/// The "select" node.
	///
	/// The node's last input denotes which region to select.
	/// All other values are passed to `start`.
	/// All values passed to `end` are output.
	Gamma,

	/// The "loop" node.
	///
	/// The node's inputs are passed to `start`.
	/// The `end`'s last input decides whether to loop.
	/// All other values are either passed to `start` or output.
	Theta,

	/// The "function" node.
	///
	/// The node's inputs are passed to `start`.
	/// The function itself is output.
	Lambda,

	/// The "mutual recursion" node.
	///
	/// The node's inputs are passed to `start`.
	/// The `end`'s inputs are passed to `start`.
	Phi,
}

impl Display for Compound {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Self::Gamma => "Gamma".fmt(f),
			Self::Theta => "Theta".fmt(f),
			Self::Lambda => "Lambda".fmt(f),
			Self::Phi => "Phi".fmt(f),
		}
	}
}

/// A node in the data flow graph.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node<S> {
	Simple(S),
	Marker(Marker),
	Compound(Compound),
}

impl<S> From<Marker> for Node<S> {
	fn from(value: Marker) -> Self {
		Self::Marker(value)
	}
}

impl<S> From<Compound> for Node<S> {
	fn from(value: Compound) -> Self {
		Self::Compound(value)
	}
}

impl<S> Node<S> {
	/// Returns a reference to the [`Node::Simple`] node if it is one.
	#[inline]
	#[must_use]
	pub const fn as_simple(&self) -> Option<&S> {
		if let Self::Simple(simple) = self {
			Some(simple)
		} else {
			None
		}
	}

	/// Returns a reference to the [`Node::Marker`] node if it is one.
	#[inline]
	#[must_use]
	pub const fn as_marker(&self) -> Option<Marker> {
		if let Self::Marker(marker) = *self {
			Some(marker)
		} else {
			None
		}
	}

	/// Returns a reference to the [`Node::Compound`] node if it is one.
	#[inline]
	#[must_use]
	pub const fn as_compound(&self) -> Option<Compound> {
		if let Self::Compound(compound) = *self {
			Some(compound)
		} else {
			None
		}
	}
}
