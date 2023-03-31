use std::collections::HashMap;

use tinyvec::TinyVec;

use crate::data_flow::{graph::Graph, node::Id};

pub type SuccessorList = TinyVec<[Id; 2]>;

/// A node successor finder.
/// It caches the successors for each node after a traversal.
#[derive(Default)]
pub struct Successors {
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
	pub fn pass<'a, S: 'a>(&'a mut self) -> impl FnMut(&Graph<S>, Id) + 'a {
		|graph, id| {
			for v in &graph.predecessors[id] {
				let successors = self.cache.entry(v.node()).or_default();

				if !successors.contains(&id) {
					successors.push(id);
				}
			}
		}
	}
}
