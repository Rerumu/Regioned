use crate::data_flow::{graph::Graph, node::NodeId};

use super::traverse::post_order::PostOrder;

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

		graph.nodes.retain(|id, _| seen.contains(&id));
	}

	/// Runs the mark-and-sweep algorithm on the given [`Graph`].
	pub fn run<S, I>(&mut self, graph: &mut Graph<S>, roots: I)
	where
		I: IntoIterator<Item = NodeId>,
	{
		self.post_order.run(graph, roots);
		self.sweep(graph);
	}
}
