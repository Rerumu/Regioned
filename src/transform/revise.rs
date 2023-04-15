use crate::{
	data_flow::{
		graph::{Graph, PredecessorList},
		link::{Link, Port},
		node::{Id, Node},
	},
	visit::successors::Successors,
};

/// Redoes the ports of the successors of the node `from` to point to the node `to`.
/// The ports are updated using the function `redo`.
pub fn redo_ports<M>(
	predecessors: &mut [PredecessorList],
	successors: &Successors,
	from: Id,
	to: Id,
	redo: M,
) where
	M: Fn(Port) -> Option<Port>,
{
	let relevant = |predecessor: &&mut Link| predecessor.node() == from;

	for &successors in &successors.cache()[from] {
		for predecessor in predecessors[successors].iter_mut().filter(relevant) {
			if let Some(port) = redo(predecessor.port()) {
				*predecessor = Link::new(to, port);
			}
		}
	}
}

/// Redoes the ports of the successors of the node `from` to point to the node `to`.
pub fn redo_ports_in_place(
	predecessors: &mut [PredecessorList],
	successors: &Successors,
	from: Id,
	to: Id,
) {
	redo_ports(predecessors, successors, from, to, Some);
}

/// Applies the `rule` to the graph nodes. If the rule succeeds,
/// the result is passed to `stitch`, the node is updated, and the old node is returned.
pub const fn single<A, B, S, U>(
	mut rule: A,
	mut stitch: B,
) -> impl FnMut(&mut Graph<S>, Id) -> Option<Node<S>>
where
	A: FnMut(&mut Graph<S>, Id) -> Option<U>,
	B: FnMut(&mut Graph<S>, Id, U) -> Node<S>,
{
	move |graph, id| {
		rule(graph, id).map(|result| {
			let node = stitch(graph, id, result);

			std::mem::replace(&mut graph.nodes[id], node)
		})
	}
}
