use slotmap::{HopSlotMap, SecondaryMap, SparseSecondaryMap};
use tinyvec::TinyVec;

use super::{
	link::Link,
	node::{Compound, Marker, Node, NodeId, Region},
};

pub type PredecessorList = TinyVec<[Link; 2]>;
pub type RegionList = TinyVec<[Region; 1]>;

/// A Regionalized Value State Dependence Graph.
///
/// It is an acyclic graph that represents the data flow of a program.
pub struct Graph<S> {
	pub nodes: HopSlotMap<NodeId, Node<S>>,
	pub regions: SparseSecondaryMap<NodeId, RegionList>,
	pub predecessors: SecondaryMap<NodeId, PredecessorList>,
}

impl<S> Graph<S> {
	/// Creates a new [`Graph`].
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Clears the graph. Keeps the allocated memory for reuse.
	pub fn clear(&mut self) {
		self.nodes.clear();
	}

	/// Adds a [`Node`] to the graph and returns its [`NodeId`].
	#[must_use]
	pub fn add_node(&mut self, node: Node<S>) -> NodeId {
		let id = self.nodes.insert(node);

		self.predecessors.insert(id, PredecessorList::default());

		id
	}

	/// Removes a [`Node`] from the graph and returns it.
	pub fn remove_node(&mut self, id: NodeId) -> Option<Node<S>> {
		self.nodes.remove(id)
	}

	/// Adds a [`Region`] to the graph and returns it.
	#[must_use]
	pub fn add_region(&mut self) -> Region {
		let start = self.add_node(Marker::Start.into());
		let end = self.add_node(Marker::End.into());

		Region::new(start, end)
	}

	/// Removes a [`Region`] from the graph.
	pub fn remove_region(&mut self, region: Region) {
		self.remove_node(region.start());
		self.remove_node(region.end());
	}

	/// Adds a [`Node`] with a [`Region`] to the graph and returns its [`NodeId`].
	#[must_use]
	pub fn add_compound(&mut self, compound: Compound) -> NodeId {
		let id = self.add_node(compound.into());
		let region = self.add_region();

		self.regions.insert(id, [region].into());

		id
	}

	/// Adds a [`Compound::Gamma`] node with [`Region`]s to the graph and returns its [`NodeId`].
	#[must_use]
	pub fn add_gamma<I>(&mut self, regions: I) -> NodeId
	where
		I: IntoIterator<Item = Region>,
	{
		let id = self.add_node(Compound::Gamma.into());
		let regions = regions.into_iter().collect();

		self.regions.insert(id, regions);

		id
	}

	/// Removes a [`Node`] with regions from the graph and returns it.
	pub fn remove_compound(&mut self, id: NodeId) -> Option<Node<S>> {
		for &region in self.regions.get(id)? {
			self.nodes.remove(region.start());
			self.nodes.remove(region.end());
		}

		self.nodes.remove(id)
	}

	/// Sets the number of predecessors for a node.
	///
	/// If it needs to be longer, it's filled with dead links. If it needs to be shorter, it's truncated.
	pub fn set_predecessor_count(&mut self, id: NodeId, count: usize) {
		self.predecessors[id].resize_with(count, Default::default);
	}

	/// Sets a connection between two nodes and ports.
	pub fn set_connection(&mut self, from: Link, to: Link) {
		self.predecessors[to.node()][to.port().index()] = from;
	}
}

impl<S> Default for Graph<S> {
	fn default() -> Self {
		Self {
			nodes: HopSlotMap::default(),
			regions: SparseSecondaryMap::default(),
			predecessors: SecondaryMap::default(),
		}
	}
}
