use crate::{
	data_flow::{link::Id, node::Parameters, nodes::Nodes},
	visit::reverse_topological::ReverseTopological,
};

/// Marks all reachable nodes from the given roots, then removes all unmarked nodes.
pub fn run<N, I>(nodes: &mut Nodes<N>, roots: I, topological: &mut ReverseTopological)
where
	N: Parameters,
	I: IntoIterator<Item = Id>,
{
	topological.iter(nodes, roots).for_each(drop);

	let seen = topological.seen();

	nodes.retain(|id, _| seen[id]);
}
