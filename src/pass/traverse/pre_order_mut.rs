use hashbrown::HashSet;

use crate::data_flow::{graph::Graph, node::Id};

/// A pre-order traversal of the graph.
/// It visits every reachable node, starting at the roots and ending at the leaves.
#[derive(Default)]
pub struct PreOrderMut {
	seen: HashSet<Id>,
	queue: Vec<(Id, bool)>,
}

impl PreOrderMut {
	/// Creates a new, reusable [`PreOrderMut`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn add_guarded(&mut self, id: Id) {
		if !self.seen.insert(id) {
			return;
		}

		self.queue.push((id, false));
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
	pub fn seen(&self) -> &HashSet<Id> {
		&self.seen
	}

	/// Traverses the graph, applying the `operation` on every node.
	/// The `operation` is allowed to modify the graph.
	pub fn run_with<S, I, O>(&mut self, graph: &mut Graph<S>, roots: I, mut operation: O)
	where
		I: IntoIterator<Item = Id>,
		O: FnMut(&mut Graph<S>, Id),
	{
		self.seen.clear();

		roots.into_iter().for_each(|id| self.add_guarded(id));

		while let Some(entry) = self.queue.last_mut() {
			let id = entry.0;

			if entry.1 {
				operation(graph, id);

				self.queue.pop();
			} else {
				entry.1 = true;

				self.add_neighbors(graph, id);
			}
		}
	}
}
