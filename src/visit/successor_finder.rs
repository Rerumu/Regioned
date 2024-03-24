use tinyvec::TinyVec;

use crate::collection::{data_flow_graph::DataFlowGraph, link::Id, node::Parameters};

use super::depth_first_searcher::DepthFirstSearcher;

pub type SuccessorList = TinyVec<[Id; 2]>;

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

		searcher.restrict(0..needed);
		searcher.run(nodes, start, |id, post| {
			if post {
				return;
			}

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
