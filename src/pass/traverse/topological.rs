use crate::data_flow::{graph::Graph, node::Id};

/// A topological traversal of the graph.
/// It visits every reachable node, starting at the roots and ending at the leaves.
#[derive(Default)]
pub struct Topological {
	seen: Vec<bool>,
	queue: Vec<Id>,
}

impl Topological {
	/// Creates a new, reusable [`Topological`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn add_guarded(&mut self, id: Id) {
		if self.seen[id] {
			return;
		}

		self.seen[id] = true;
		self.queue.push(id);
	}

	fn add_neighbors<S>(&mut self, graph: &Graph<S>, id: Id) {
		if graph.nodes[id].as_compound().is_some() {
			for region in &graph.regions[&id] {
				self.add_guarded(region.start());
				self.add_guarded(region.end());
			}
		}

		for link in &graph.predecessors[id] {
			self.add_guarded(link.node());
		}
	}

	/// Returns the nodes that have been seen.
	#[must_use]
	pub fn seen(&self) -> &[bool] {
		&self.seen
	}

	/// Traverses the graph, applying the `operation` on every node.
	pub fn run_with<S, I, O>(&mut self, graph: &Graph<S>, roots: I, mut operation: O)
	where
		I: IntoIterator<Item = Id>,
		O: FnMut(&Graph<S>, Id),
	{
		self.seen.clear();
		self.seen.resize(graph.active(), false);

		roots.into_iter().for_each(|id| self.add_guarded(id));

		while let Some(id) = self.queue.pop() {
			operation(graph, id);

			self.add_neighbors(graph, id);
		}
	}

	/// Traverses the graph. Performs no operation on the nodes.
	pub fn run<S, I>(&mut self, graph: &Graph<S>, roots: I)
	where
		I: IntoIterator<Item = Id>,
	{
		self.run_with(graph, roots, |_, _| {});
	}
}
