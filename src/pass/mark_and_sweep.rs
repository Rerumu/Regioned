use crate::data_flow::{graph::Graph, node::NodeId};

use super::walker::Walker;

#[derive(Default)]
pub struct MarkAndSweep {
	walker: Walker,
}

impl MarkAndSweep {
	/// Creates a new, reusable [`MarkAndSweep`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn sweep<S>(&self, graph: &mut Graph<S>) {
		let seen = self.walker.seen();

		graph.nodes.retain(|id, _| seen.contains(&id));
	}

	/// Runs the mark-and-sweep algorithm on the given [`Graph`].
	pub fn run<S, I>(&mut self, graph: &mut Graph<S>, roots: I)
	where
		I: IntoIterator<Item = NodeId>,
	{
		self.walker.run(graph, roots);
		self.sweep(graph);
	}
}
