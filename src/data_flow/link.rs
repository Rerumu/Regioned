use std::num::TryFromIntError;

use super::node::Id;

/// An index of a port to either an input or output.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Port {
	index: u16,
}

impl Port {
	/// Creates a new [`Port`] from an index.
	#[inline]
	#[must_use]
	pub const fn new(index: u16) -> Self {
		Self { index }
	}

	/// Returns the index of the [`Port`].
	#[inline]
	#[must_use]
	pub const fn index(self) -> u16 {
		self.index
	}

	/// Returns an iterator over all [`Port`]s starting from the current one.
	pub fn iter(self) -> impl Iterator<Item = Self> {
		(self.index..).map(Self::new)
	}
}

impl From<Port> for usize {
	#[inline]
	fn from(port: Port) -> Self {
		port.index.into()
	}
}

impl TryFrom<usize> for Port {
	type Error = TryFromIntError;

	#[inline]
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		u16::try_from(value).map(|index| Self { index })
	}
}

/// A relationship between an [`Id`] and a [`Port`]. Two of these together
/// represent a connection between two nodes.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
	node: Id,
	port: Port,
}

impl Link {
	/// Creates a new [`Link`] from an [`Id`] and a [`Port`].
	#[inline]
	#[must_use]
	pub const fn new(node: Id, port: Port) -> Self {
		Self { node, port }
	}

	/// Returns the [`Id`] of the [`Link`].
	#[inline]
	#[must_use]
	pub const fn node(self) -> Id {
		self.node
	}

	/// Returns the [`Port`] of the [`Link`].
	#[inline]
	#[must_use]
	pub const fn port(self) -> Port {
		self.port
	}

	/// Returns an iterator over all [`Link`]s starting from the current one.
	pub fn iter(self) -> impl Iterator<Item = Self> {
		self.port.iter().map(move |p| Self::new(self.node, p))
	}
}

impl From<Id> for Link {
	#[inline]
	fn from(node: Id) -> Self {
		Self::new(node, Port::default())
	}
}
