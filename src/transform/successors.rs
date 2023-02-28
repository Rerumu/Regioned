use std::collections::HashMap;

use tinyvec::TinyVec;

use crate::data_flow::{graph::Graph, node::NodeId};

use super::walker::Walker;

pub type SuccessorList = TinyVec<[NodeId; 2]>;

#[derive(Default)]
pub struct Successors {
	walker: Walker,
	cache: HashMap<NodeId, SuccessorList>,
}

impl Successors {
	/// Creates a new, reusable [`Successors`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Returns the cached successors.
	#[must_use]
	pub fn cache(&self) -> &HashMap<NodeId, SuccessorList> {
		&self.cache
	}

	/// Finds all successors coming back from the roots.
	pub fn run<S, I>(&mut self, graph: &Graph<S>, roots: I)
	where
		I: IntoIterator<Item = NodeId>,
	{
		self.walker.run_with(graph, roots, |id| {
			for v in &graph.predecessors[id] {
				let successors = self.cache.entry(v.node()).or_default();

				if !successors.contains(&id) {
					successors.push(id);
				}
			}
		});
	}
}
