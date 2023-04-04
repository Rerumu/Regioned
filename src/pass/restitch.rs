use crate::data_flow::{
	graph::Graph,
	node::{Id, Node},
};

/// Applies the rule `applier` to the graph nodes.
/// If the rule succeeds, the result is passed to `stitcher` and the node is updated.
pub const fn pass<A, B, S, U>(mut applier: A, mut stitcher: B) -> impl FnMut(&mut Graph<S>, Id)
where
	A: FnMut(&mut Graph<S>, Id) -> Option<U>,
	B: FnMut(&mut Graph<S>, Id, U) -> Node<S>,
{
	move |graph, id| {
		if let Some(result) = applier(graph, id) {
			graph.nodes[id] = stitcher(graph, id, result);
		}
	}
}
