use arena::collection::Arena;
use hashbrown::HashMap;
use tinyvec::TinyVec;

use super::{
	link::Link,
	node::{Compound, Id, Marker, Node, Region},
};

pub use arena::key::Key;

pub type PredecessorList = TinyVec<[Link; 3]>;
pub type RegionList = TinyVec<[Region; 2]>;

/// A Regionalized Value State Dependence Graph.
///
/// It is an acyclic graph that represents the data flow of a program.
pub struct Graph<S> {
	pub nodes: Arena<Id, Node<S>>,
	pub regions: HashMap<Id, RegionList>,
	pub predecessors: Vec<PredecessorList>,
}

impl<S> Graph<S> {
	/// Creates a new, empty [`Graph`].
	#[inline]
	#[must_use]
	pub fn new() -> Self {
		Self {
			nodes: Arena::new(),
			regions: HashMap::new(),
			predecessors: Vec::new(),
		}
	}

	/// Creates a new, empty [`Graph`] with the specified capacity.
	#[inline]
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			nodes: Arena::with_capacity(capacity),
			regions: HashMap::new(),
			predecessors: Vec::with_capacity(capacity),
		}
	}

	/// Returns the total number of active indices in the [`Graph`].
	#[inline]
	#[must_use]
	pub fn active(&self) -> usize {
		self.nodes.keys().next_back().map_or(0, |id| id.index() + 1)
	}

	/// Clears the graph. Keeps the allocated memory for reuse.
	#[inline]
	pub fn clear(&mut self) {
		self.nodes.clear();
		self.regions.clear();
	}

	/// Adds a [`Node`] to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_node(&mut self, node: Node<S>) -> Id {
		let id = self.nodes.insert(node);

		if let Some(last) = self.predecessors.get_mut(id.index()) {
			*last = PredecessorList::new();
		} else {
			self.predecessors.push(PredecessorList::new());
		}

		id
	}

	/// Removes a [`Node`] from the graph and returns it.
	#[inline]
	pub fn remove_node(&mut self, id: Id) -> Option<Node<S>> {
		self.nodes.try_remove(id)
	}

	/// Adds a [`Region`] to the graph and returns it.
	#[inline]
	#[must_use]
	pub fn add_region(&mut self) -> Region {
		let start = self.add_node(Marker::Start.into());
		let end = self.add_node(Marker::End.into());

		Region::new(start, end)
	}

	/// Removes a [`Region`] from the graph.
	#[inline]
	pub fn remove_region(&mut self, region: Region) {
		self.nodes.remove(region.start());
		self.nodes.remove(region.end());
	}

	/// Adds a [`Node::Compound`] to the graph and returns its [`Id`] and [`Region`].
	#[inline]
	#[must_use]
	pub fn add_compound(&mut self, compound: Compound) -> (Id, Region) {
		let id = self.add_node(compound.into());
		let region = self.add_region();

		self.regions.insert(id, std::iter::once(region).collect());

		(id, region)
	}

	/// Adds a [`Compound::Gamma`] node with the [`RegionList`] to the graph and returns its [`Id`].
	#[inline]
	#[must_use]
	pub fn add_gamma(&mut self, regions: RegionList) -> Id {
		let id = self.add_node(Compound::Gamma.into());

		self.regions.insert(id, regions);

		id
	}

	/// Removes a [`Node::Compound`] with regions from the graph and returns it.
	#[inline]
	pub fn remove_compound(&mut self, id: Id) -> Option<Compound> {
		for region in self.regions.remove(&id)? {
			self.remove_region(region);
		}

		self.nodes
			.try_remove(id)
			.as_ref()
			.and_then(Node::as_compound)
	}
}

impl<S> Default for Graph<S> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}
