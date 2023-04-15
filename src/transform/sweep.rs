use crate::{
	data_flow::{graph::Graph, node::Id},
	visit::reverse_topological::ReverseTopological,
};

/// Marks all reachable nodes from the given roots, then removes all unmarked nodes.
pub fn run<S, I>(graph: &mut Graph<S>, roots: I, topological: &mut ReverseTopological)
where
	I: IntoIterator<Item = Id>,
{
	topological.run(graph, roots);

	let seen = topological.seen();

	graph.nodes.retain(|id, _| seen[id]);
	graph.regions.retain(|id, _| seen[*id]);
}
