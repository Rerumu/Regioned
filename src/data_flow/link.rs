use std::num::TryFromIntError;

use super::node::NodeId;

/// An index of a port to either an input or output.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Port {
	index: u16,
}

impl Port {
	/// Creates a new [`Port`] from an index.
	#[must_use]
	pub const fn new(index: u16) -> Self {
		Self { index }
	}

	/// Returns the index of the [`Port`].
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
	fn from(port: Port) -> Self {
		port.index.into()
	}
}

impl TryFrom<usize> for Port {
	type Error = TryFromIntError;

	fn try_from(value: usize) -> Result<Self, Self::Error> {
		u16::try_from(value).map(|index| Self { index })
	}
}

/// A relationship between a [`NodeId`] and a [`Port`]. Two of these together
/// represent a connection between two nodes.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Link {
	node: NodeId,
	port: Port,
}

impl Link {
	/// Creates a new [`Link`] from a [`NodeId`] and a [`Port`].
	#[must_use]
	pub const fn new(node: NodeId, port: Port) -> Self {
		Self { node, port }
	}

	/// Returns the [`NodeId`] of the [`Link`].
	#[must_use]
	pub const fn node(self) -> NodeId {
		self.node
	}

	/// Returns the [`Port`] of the [`Link`].
	#[must_use]
	pub const fn port(self) -> Port {
		self.port
	}

	/// Returns an iterator over all [`Link`]s starting from the current one.
	pub fn iter(self) -> impl Iterator<Item = Self> {
		self.port.iter().map(move |p| Link::new(self.node, p))
	}
}
