use super::{
	link::{Id, Link},
	node::{Gamma, Lambda, Node, Phi, Theta},
};

pub struct DataFlowGraph<T> {
	nodes: Vec<Node<T>>,
}

impl<T> DataFlowGraph<T> {
	/// Creates a new, empty graph.
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self { nodes: Vec::new() }
	}

	/// Returns a reference to the inner [`Vec`] of the graph.
	#[inline]
	#[must_use]
	pub const fn nodes(&self) -> &Vec<Node<T>> {
		&self.nodes
	}

	/// Returns a mutable reference to the inner [`Vec`] of the graph.
	#[inline]
	#[must_use]
	pub fn nodes_mut(&mut self) -> &mut Vec<Node<T>> {
		&mut self.nodes
	}

	fn add_node(&mut self, node: Node<T>) -> Id {
		let position = self.nodes.len();

		self.nodes.push(node);

		super::link::Id::from_usize(position)
	}

	/// Adds a [`Node::Simple`] node to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_simple<U: Into<T>>(&mut self, data: U) -> Id {
		let node = Node::Simple(data.into());

		self.add_node(node)
	}

	/// Adds a [`Node::Simple`] node to the graph and returns its [`Link`].
	#[inline]
	#[must_use]
	pub fn add_simple_at<U: Into<T>>(&mut self, data: U, port: u16) -> Link {
		let id = self.add_simple(data);

		Link(id, port)
	}

	/// Adds a [`Node::Gamma`] node to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_gamma(&mut self, parameters: Vec<Link>, results: Vec<Vec<Link>>) -> Id {
		let node = Node::Gamma(Gamma {
			parameters,
			results,
		});

		self.add_node(node)
	}

	/// Adds a [`Node::Theta`] node to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_theta(&mut self, parameters: Vec<Link>, results: Vec<Link>) -> Id {
		let node = Node::Theta(Theta {
			parameters,
			results,
		});

		self.add_node(node)
	}

	/// Adds a [`Node::Phi`] node to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_phi(&mut self, parameters: Vec<Link>, results: Vec<Link>) -> Id {
		let node = Node::Phi(Phi {
			parameters,
			results,
		});

		self.add_node(node)
	}

	/// Adds a [`Node::Lambda`] node to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_lambda(&mut self, parameters: Vec<Link>, results: Vec<Link>) -> Id {
		let node = Node::Lambda(Lambda {
			parameters,
			results,
		});

		self.add_node(node)
	}
}

impl<T> Default for DataFlowGraph<T> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

impl<T> std::ops::Deref for DataFlowGraph<T> {
	type Target = [Node<T>];

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
