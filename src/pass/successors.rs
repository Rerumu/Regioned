use std::collections::HashMap;

use arena::key::Key;
use tinyvec::TinyVec;

use crate::data_flow::{graph::Graph, node::Id};

use super::traverse::post_order::PostOrder;

pub type SuccessorList = TinyVec<[Id; 2]>;

/// A node successor finder.
/// It caches the successors for each node after a traversal.
#[derive(Default)]
pub struct Successors {
	post_order: PostOrder,
	cache: HashMap<Id, SuccessorList>,
}

impl Successors {
	/// Creates a new, reusable [`Successors`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Returns the cached successors.
	#[must_use]
	pub fn cache(&self) -> &HashMap<Id, SuccessorList> {
		&self.cache
	}

	/// Finds all successors coming back from the roots.
	pub fn run<S, I>(&mut self, graph: &Graph<S>, roots: I)
	where
		I: IntoIterator<Item = Id>,
	{
		self.post_order.run_with(graph, roots, |id| {
			for v in &graph.predecessors[id.index()] {
				let successors = self.cache.entry(v.node()).or_default();

				if !successors.contains(&id) {
					successors.push(id);
				}
			}
		});
	}
}
