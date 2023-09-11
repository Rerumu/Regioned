use std::ops::{Deref, DerefMut};

use arena::{collection::Arena, key::Key};
use tinyvec::TinyVec;

use super::{
	link::{Id, Link, Region},
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

	/// Adds a [`Node::Simple`] node to the graph and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_simple<I: Into<N>>(&mut self, data: I) -> Link {
		let simple = Node::Simple(data.into());

		self.nodes.insert(simple).into()
	}

	/// Adds a [`Region`] to the graph and returns it.
	#[inline]
	#[must_use]
	pub fn add_region<F>(&mut self, results: F) -> Region
	where
		F: FnOnce(&mut Self, Link) -> Vec<Link>,
	{
		let start = self.nodes.insert(Node::Marker(Marker::Start));
		let end = Node::Marker(Marker::End {
			parameters: results(self, start.into()),
		});

		Region {
			start,
			end: self.nodes.insert(end),
		}
	}

	/// Adds a [`Compound::Gamma`] node to the and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_gamma(&mut self, parameters: Vec<Link>, regions: TinyVec<[Region; 2]>) -> Link {
		let compound = Node::Compound(Compound::Gamma {
			parameters,
			regions,
		});

		self.nodes.insert(compound).into()
	}

	/// Adds a [`Compound::Theta`] node to the and returns its [`Link`] and [`Region`].
	#[inline]
	#[must_use]
	pub fn add_theta<F>(&mut self, parameters: Vec<Link>, results: F) -> (Link, Region)
	where
		F: FnOnce(&mut Self, Link) -> Vec<Link>,
	{
		let region = self.add_region(results);
		let compound = Node::Compound(Compound::Theta { parameters, region });

		(self.nodes.insert(compound).into(), region)
	}

	/// Adds a [`Compound::Lambda`] node to the and returns its [`Link`] and [`Region`].
	#[inline]
	#[must_use]
	pub fn add_lambda<F>(&mut self, parameters: Vec<Link>, results: F) -> (Link, Region)
	where
		F: FnOnce(&mut Self, Link) -> Vec<Link>,
	{
		let region = self.add_region(results);
		let compound = Node::Compound(Compound::Lambda { parameters, region });

		(self.nodes.insert(compound).into(), region)
	}

	/// Adds a [`Compound::Phi`] node to the and returns its [`Link`] and [`Region`].
	#[inline]
	#[must_use]
	pub fn add_phi<F>(&mut self, parameters: Vec<Link>, results: F) -> (Link, Region)
	where
		F: FnOnce(&mut Self, Link) -> Vec<Link>,
	{
		let region = self.add_region(results);
		let compound = Node::Compound(Compound::Phi { parameters, region });

		(self.nodes.insert(compound).into(), region)
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
