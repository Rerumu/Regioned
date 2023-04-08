use crate::data_flow::{
	graph::{Graph, PredecessorList},
	node::{Id, Region},
};

enum Entry {
	Predecessors { id: Id, count: usize },
	Regions { id: Id, count: usize },
	Node { id: Id },
}

fn region_count_checked<S>(graph: &Graph<S>, id: Id) -> usize {
	graph.nodes[id]
		.as_compound()
		.map_or(0, |_| graph.regions[&id].len())
}

/// A reverse topological traversal of the graph.
/// It visits nodes starting from the leaves in the order `Predecessors 0 -> N, Regions 0 -> N, Node`.
#[derive(Default)]
pub struct ReverseTopological {
	seen: Vec<bool>,
	stack: Vec<Entry>,
}

impl ReverseTopological {
	/// Creates a new, reusable [`ReverseTopological`] instance.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			seen: Vec::new(),
			stack: Vec::new(),
		}
	}

	/// Returns the nodes that have been seen.
	#[must_use]
	pub fn seen(&self) -> &[bool] {
		&self.seen
	}

	fn add_node_by_id(&mut self, predecessors: &[PredecessorList], id: Id) {
		if self.seen[id] {
			return;
		}

		let count = predecessors[id].len();

		self.seen[id] = true;
		self.stack.push(Entry::Predecessors { id, count });
	}

	fn add_region(&mut self, predecessors: &[PredecessorList], region: Region) {
		self.add_node_by_id(predecessors, region.end());
		self.add_node_by_id(predecessors, region.start());
	}

	fn handle_predecessor<S>(&mut self, graph: &Graph<S>, count: usize, id: Id) {
		if let Some(count) = count.checked_sub(1) {
			self.stack.push(Entry::Predecessors { id, count });

			let predecessors = &graph.predecessors[id];
			let predecessor = predecessors[predecessors.len() - count - 1];

			self.add_node_by_id(&graph.predecessors, predecessor.node());
		} else {
			let count = region_count_checked(graph, id);

			self.stack.push(Entry::Regions { id, count });
		}
	}

	fn handle_region<S>(&mut self, graph: &Graph<S>, count: usize, id: Id) {
		if let Some(count) = count.checked_sub(1) {
			self.stack.push(Entry::Regions { id, count });

			let regions = &graph.regions[&id];
			let region = regions[regions.len() - count - 1];

			self.add_region(&graph.predecessors, region);
		} else {
			self.stack.push(Entry::Node { id });
		}
	}

	fn set_up_roots<I>(&mut self, predecessors: &[PredecessorList], active: usize, roots: I)
	where
		I: IntoIterator<Item = Id>,
	{
		self.seen.clear();
		self.seen.resize(active, false);

		for id in roots {
			self.add_node_by_id(predecessors, id);
		}

		self.stack.reverse();
	}

	/// Traverses the graph and logs every seen node.
	pub fn run<S, I>(&mut self, graph: &Graph<S>, roots: I)
	where
		I: IntoIterator<Item = Id>,
	{
		self.run_with(graph, roots, |_, _| {});
	}

	/// Traverses the graph, applying the `function` on every node.
	pub fn run_with<S, I, F>(&mut self, graph: &Graph<S>, roots: I, mut function: F)
	where
		I: IntoIterator<Item = Id>,
		F: FnMut(&Graph<S>, Id),
	{
		self.set_up_roots(&graph.predecessors, graph.active(), roots);

		while let Some(entry) = self.stack.pop() {
			match entry {
				Entry::Predecessors { id, count } => self.handle_predecessor(graph, count, id),
				Entry::Regions { id, count } => self.handle_region(graph, count, id),
				Entry::Node { id } => function(graph, id),
			}
		}
	}

	/// Traverses the graph, applying the `function` on every node.
	/// The `function` is allowed to modify the graph.
	pub fn run_with_mut<S, I, F>(&mut self, graph: &mut Graph<S>, roots: I, mut function: F)
	where
		I: IntoIterator<Item = Id>,
		F: FnMut(&mut Graph<S>, Id),
	{
		self.set_up_roots(&graph.predecessors, graph.active(), roots);

		while let Some(entry) = self.stack.pop() {
			match entry {
				Entry::Predecessors { id, count } => self.handle_predecessor(graph, count, id),
				Entry::Regions { id, count } => self.handle_region(graph, count, id),
				Entry::Node { id } => function(graph, id),
			}
		}
	}
}
