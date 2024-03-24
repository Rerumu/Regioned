use arena::collection::Arena;
use tinyvec::TinyVec;

use super::{
	link::{Id, Link},
	node::{Gamma, Lambda, Node, Phi, Theta},
};

pub struct DataFlowGraph<T> {
	nodes: Arena<Id, Node<T>>,
}

impl<T> DataFlowGraph<T> {
	/// Creates a new, empty graph.
	#[inline]
	#[must_use]
	pub fn new() -> Self {
		let nodes = Arena::new();

		Self { nodes }
	}

	/// Creates a new, empty graph with the specified capacity.
	#[inline]
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		let nodes = Arena::with_capacity(capacity);

		Self { nodes }
	}

	/// Returns a reference to the inner [`Arena`] of the graph.
	#[inline]
	#[must_use]
	pub const fn nodes(&self) -> &Arena<Id, Node<T>> {
		&self.nodes
	}

	/// Returns a mutable reference to the inner [`Arena`] of the graph.
	#[inline]
	#[must_use]
	pub fn nodes_mut(&mut self) -> &mut Arena<Id, Node<T>> {
		&mut self.nodes
	}

	/// Adds a [`Node::Simple`] node to the graph and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_simple<U: Into<T>>(&mut self, data: U) -> Link {
		let node = Node::Simple(data.into());

		self.nodes.insert(node).into()
	}

	/// Adds a [`Node::Gamma`] node to the graph and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_gamma(&mut self, parameters: Vec<Link>, results: TinyVec<[Vec<Link>; 2]>) -> Link {
		let node = Node::Gamma(Gamma {
			parameters,
			results,
		});

		self.nodes.insert(node).into()
	}

	/// Adds a [`Node::Theta`] node to the graph and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_theta(&mut self, parameters: Vec<Link>, results: Vec<Link>) -> Link {
		let node = Node::Theta(Theta {
			parameters,
			results,
		});

		self.nodes.insert(node).into()
	}

	/// Adds a [`Node::Phi`] node to the graph and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_phi(&mut self, parameters: Vec<Link>, results: Vec<Link>) -> Link {
		let node = Node::Phi(Phi {
			parameters,
			results,
		});

		self.nodes.insert(node).into()
	}

	/// Adds a [`Node::Lambda`] node to the graph and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_lambda(&mut self, parameters: Vec<Link>, results: Vec<Link>) -> Link {
		let node = Node::Lambda(Lambda {
			parameters,
			results,
		});

		self.nodes.insert(node).into()
	}
}

impl<T> Default for DataFlowGraph<T> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

impl<T> std::ops::Deref for DataFlowGraph<T> {
	type Target = Arena<Id, Node<T>>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.nodes()
	}
}

impl<T> std::ops::DerefMut for DataFlowGraph<T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.nodes_mut()
	}
}
