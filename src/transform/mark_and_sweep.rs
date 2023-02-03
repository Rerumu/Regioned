use std::collections::HashSet;

use crate::data_flow::{graph::Graph, node::NodeId};

#[derive(Default)]
pub struct MarkAndSweep {
	marked: HashSet<NodeId>,
	boundary: Vec<NodeId>,
}

impl MarkAndSweep {
	/// Creates a new, reusable [`MarkAndSweep`] instance.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn mark_node_at(&mut self, id: NodeId) {
		if self.marked.contains(&id) {
			return;
		}

		self.marked.insert(id);
		self.boundary.push(id);
	}

	fn mark_links_at<S>(&mut self, graph: &Graph<S>, id: NodeId) {
		if graph.nodes[id].as_compound().is_some() {
			for region in &graph.regions[id] {
				self.mark_node_at(region.start());
				self.mark_node_at(region.end());
			}
		}

		for link in &graph.predecessors[id] {
			self.mark_node_at(link.node());
		}
	}

	fn mark<S>(&mut self, graph: &Graph<S>, roots: &[NodeId]) {
		for &root in roots {
			self.mark_node_at(root);
		}

		while let Some(id) = self.boundary.pop() {
			self.mark_links_at(graph, id);
		}
	}

	fn sweep<S>(&self, graph: &mut Graph<S>) {
		graph.nodes.retain(|id, _| self.marked.contains(&id));
	}

	/// Runs the mark-and-sweep algorithm on the given [`Graph`].
	pub fn run<S>(&mut self, graph: &mut Graph<S>, roots: &[NodeId]) {
		self.marked.clear();

		self.mark(graph, roots);
		self.sweep(graph);
	}
}
