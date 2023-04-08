use crate::data_flow::{graph::Graph, node::Id};

use super::traverse::reverse_topological::ReverseTopological;

/// A mark-and-sweep algorithm for removing unreachable nodes from the graph.
/// It uses a shallow traversal which may result in logically unreachable nodes being retained.
#[derive(Default)]
pub struct MarkAndSweep {
	topological: ReverseTopological,
}

impl MarkAndSweep {
	/// Creates a new, reusable [`MarkAndSweep`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn sweep<S>(&self, graph: &mut Graph<S>) {
		let seen = self.topological.seen();

		graph.nodes.retain(|id, _| seen[id]);
		graph.regions.retain(|id, _| seen[*id]);
	}

	/// Removes unreachable nodes from the graph.
	pub fn run<S, I>(&mut self, graph: &mut Graph<S>, roots: I)
	where
		I: IntoIterator<Item = Id>,
	{
		self.topological.run(graph, roots);
		self.sweep(graph);
	}
}
