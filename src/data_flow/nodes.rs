use std::ops::{Deref, DerefMut};

use arena::{collection::Arena, key::Key};
use tinyvec::TinyVec;

use super::{
	link::{Id, Region},
	node::{Compound, Marker, Node},
};

/// A Regionalized Value State Dependence Graph.
///
/// It is an acyclic graph that represents the data flow of a program.
#[derive(Clone, Debug)]
pub struct Nodes<N> {
	nodes: Arena<Id, Node<N>>,
}

impl<N> Nodes<N> {
	/// Creates a new, empty [`Nodes`].
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		let nodes = Arena::new();

		Self { nodes }
	}

	/// Creates a new, empty [`Nodes`] with the specified capacity.
	#[inline]
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		let nodes = Arena::with_capacity(capacity);

		Self { nodes }
	}

	/// Returns the number of active nodes in the graph.
	#[inline]
	#[must_use]
	pub fn active(&self) -> usize {
		self.nodes.keys().next_back().map_or(0, |id| id.index() + 1)
	}

	/// Adds a [`Node::Simple`] node to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_simple(&mut self, simple: N) -> Id {
		self.nodes.insert(Node::Simple(simple))
	}

	/// Adds a [`Region`] to the graph and returns it.
	#[inline]
	#[must_use]
	pub fn add_region(&mut self) -> Region {
		let start = self.nodes.insert(Node::Marker(Marker::Start));
		let end = self.nodes.insert(Node::Marker(Marker::End {
			parameters: Vec::new(),
		}));

		Region::new(start, end)
	}

	/// Adds a [`Compound::Gamma`] node to the and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_gamma(&mut self, regions: TinyVec<[Region; 2]>) -> Id {
		let compound = Compound::Gamma {
			parameters: Vec::new(),
			regions,
		};

		self.nodes.insert(compound.into())
	}

	/// Adds a [`Compound::Theta`] node to the and returns its [`Id`] and [`Region`].
	#[inline]
	#[must_use]
	pub fn add_theta(&mut self) -> (Id, Region) {
		let region = self.add_region();
		let compound = Compound::Theta {
			parameters: Vec::new(),
			region,
		};

		(self.nodes.insert(compound.into()), region)
	}

	/// Adds a [`Compound::Lambda`] node to the and returns its [`Id`] and [`Region`].
	#[inline]
	#[must_use]
	pub fn add_lambda(&mut self) -> (Id, Region) {
		let region = self.add_region();
		let compound = Compound::Lambda {
			parameters: Vec::new(),
			region,
		};

		(self.nodes.insert(compound.into()), region)
	}

	/// Adds a [`Compound::Phi`] node to the and returns its [`Id`] and [`Region`].
	#[inline]
	#[must_use]
	pub fn add_phi(&mut self) -> (Id, Region) {
		let region = self.add_region();
		let compound = Compound::Phi {
			parameters: Vec::new(),
			region,
		};

		(self.nodes.insert(compound.into()), region)
	}
}

impl<N> Default for Nodes<N> {
	#[inline]
	#[must_use]
	fn default() -> Self {
		Self::new()
	}
}

impl<N> Deref for Nodes<N> {
	type Target = Arena<Id, Node<N>>;

	#[inline]
	#[must_use]
	fn deref(&self) -> &Self::Target {
		&self.nodes
	}
}

impl<N> DerefMut for Nodes<N> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.nodes
	}
}
