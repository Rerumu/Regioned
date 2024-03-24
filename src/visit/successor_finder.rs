use list::resizable::Resizable;

use crate::collection::{data_flow_graph::DataFlowGraph, link::Id, node::Parameters};

pub struct SuccessorFinder {
	cache: Vec<Resizable<Id, 3>>,
}

impl SuccessorFinder {
	/// Creates a new, reusable [`SuccessorFinder`] instance.
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self { cache: Vec::new() }
	}

	/// Returns a slice of the successors of `id`.
	#[inline]
	#[must_use]
	pub fn at(&self, id: Id) -> &[Id] {
		&self.cache[id]
	}

	/// Clears the cache.
	#[inline]
	pub fn clear(&mut self) {
		self.cache.clear();
	}

	fn set_capacity(&mut self, capacity: usize) {
		self.cache.iter_mut().for_each(Resizable::clear);

		if self.cache.len() < capacity {
			self.cache.resize_with(capacity, Resizable::new);
		}
	}

	fn set_parent(&mut self, predecessor: Id, parent: Id) {
		let successors = &mut self.cache[predecessor];

		if successors.first() != Some(&parent) {
			successors.insert(0, parent);
		}
	}

	fn add_successor(&mut self, predecessor: Id, successor: Id) {
		let successors = &mut self.cache[predecessor];

		if !successors.contains(&successor) {
			successors.push(successor);
		}
	}

	fn find_all_successors<T: Parameters>(&mut self, data_flow_graph: &DataFlowGraph<T>) {
		for (index, node) in data_flow_graph.iter().enumerate() {
			let id = Id::from_usize(index);

			if let Some(results_list) = node.as_results() {
				for result in results_list.iter().flatten() {
					self.set_parent(result.0, id);
				}
			}

			for predecessor in node.parameters() {
				self.add_successor(predecessor.0, id);
			}
		}
	}

	/// Finds all successors for the nodes in the graph.
	pub fn run<T>(&mut self, data_flow_graph: &DataFlowGraph<T>)
	where
		T: Parameters,
	{
		let needed = data_flow_graph.nodes().len();

		self.set_capacity(needed);
		self.find_all_successors(data_flow_graph);
	}
}

impl Default for SuccessorFinder {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}
