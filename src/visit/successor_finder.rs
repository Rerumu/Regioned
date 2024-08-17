use list::resizable::Resizable;

use crate::collection::{data_flow_graph::DataFlowGraph, link::Id, node::Parameters};

use super::depth_first_searcher::{DepthFirstSearcher, Event};

pub type SuccessorList = Resizable<Id, 2>;

/// A node successor finder.
/// It caches the successors for each node after a traversal.
pub struct SuccessorFinder {
	cache: Vec<SuccessorList>,
}

impl SuccessorFinder {
	/// Creates a new, reusable [`Successors`] instance.
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self { cache: Vec::new() }
	}

	/// Returns the cached successors.
	#[must_use]
	pub fn cache(&self) -> &[SuccessorList] {
		&self.cache
	}

	/// Clears the cache.
	pub fn clear(&mut self) {
		self.cache.clear();
	}

	/// Finds and caches all successors coming back from the start.
	pub fn run<T>(&mut self, nodes: &DataFlowGraph<T>, start: Id, searcher: &mut DepthFirstSearcher)
	where
		T: Parameters,
	{
		let needed = nodes.indices_needed();

		self.cache.iter_mut().for_each(SuccessorList::clear);

		if self.cache.len() < needed {
			self.cache.resize_with(needed, SuccessorList::new);
		}

		let active = searcher.nodes_mut();

		active.clear();
		active.extend(0..needed);

		searcher.run(nodes, start, |event| {
			let Event::PreNode { id } = event else { return };

			for predecessor in nodes[id].parameters() {
				let successors = &mut self.cache[predecessor.node];

				if !successors.contains(&id) {
					successors.push(id);
				}
			}
		});
	}
}

impl Default for SuccessorFinder {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}
