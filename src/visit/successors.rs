use tinyvec::TinyVec;

use crate::data_flow::{graph::Graph, node::Id};

use super::reverse_topological::ReverseTopological;

pub type SuccessorList = TinyVec<[Id; 2]>;

/// A node successor finder.
/// It caches the successors for each node after a traversal.
#[derive(Default)]
pub struct Successors {
	cache: Vec<SuccessorList>,
}

impl Successors {
	/// Creates a new, reusable [`Successors`] instance.
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

	/// Finds and caches all successors coming back from the roots.
	pub fn run<S, I>(&mut self, graph: &Graph<S>, roots: I, topological: &mut ReverseTopological)
	where
		I: IntoIterator<Item = Id>,
	{
		let active = graph.active();

		self.cache.iter_mut().for_each(SuccessorList::clear);

		if self.cache.len() < active {
			self.cache.resize_with(active, SuccessorList::new);
		}

		topological.run_with(graph, roots, |graph, id| {
			for predecessor in &graph.predecessors[id] {
				let successors = &mut self.cache[predecessor.node()];

				if !successors.contains(&id) {
					successors.push(id);
				}
			}
		});
	}
}
