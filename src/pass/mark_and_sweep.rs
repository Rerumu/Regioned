use crate::data_flow::{graph::Graph, node::Id};

use super::traverse::post_order::PostOrder;

/// A mark-and-sweep algorithm for removing unreachable nodes from the graph.
/// It uses a shallow traversal which may result in logically unreachable nodes being retained.
#[derive(Default)]
pub struct MarkAndSweep {
	post_order: PostOrder,
}

impl MarkAndSweep {
	/// Creates a new, reusable [`MarkAndSweep`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn sweep<S>(&self, graph: &mut Graph<S>) {
		let seen = self.post_order.seen();

		graph.nodes.retain(|id, _| seen[id]);
		graph.regions.retain(|id, _| seen[*id]);
	}

	/// Removes unreachable nodes from the graph.
	pub fn run<S, I>(&mut self, graph: &mut Graph<S>, roots: I)
	where
		I: IntoIterator<Item = Id>,
	{
		self.post_order.run(graph, roots);
		self.sweep(graph);
	}
}
