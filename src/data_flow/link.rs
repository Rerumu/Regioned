use super::node::NodeId;

/// An index of a port to either an input or output.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Port {
	index: u16,
}

impl Port {
	/// Creates a new [`Port`] from an index.
	#[must_use]
	pub fn new(index: usize) -> Self {
		let index = u16::try_from(index).expect("Port index too large");

		Self { index }
	}

	/// Returns the index of the [`Port`].
	#[must_use]
	pub fn index(self) -> usize {
		self.index.into()
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
}
