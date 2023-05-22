use crate::{
	data_flow::{
		link::{Id, Link},
		node::{Node, ParametersMut},
		nodes::Nodes,
	},
	visit::successors::Successors,
};

/// Redoes the ports of the successors of the node `id` to point elsewhere.
/// The ports are updated using the function `redo`.
pub fn redo_ports<S, F>(nodes: &mut Nodes<S>, successors: &Successors, id: Id, redo: F) -> usize
where
	S: ParametersMut,
	F: Fn(u16) -> Option<Link>,
{
	let mut applied = 0;

	for &successor in &successors.cache()[id] {
		for predecessor in nodes[successor].parameters_mut() {
			if predecessor.node == id {
				if let Some(link) = redo(predecessor.port) {
					*predecessor = link;

					applied += 1;
				}
			}
		}
	}

	applied
}

/// Redoes the ports of the successors of the node `from` to point to the node `to`.
pub fn redo_ports_in_place<S>(
	nodes: &mut Nodes<S>,
	successors: &Successors,
	from: Id,
	to: Id,
) -> usize
where
	S: ParametersMut,
{
	redo_ports(nodes, successors, from, |port| {
		Some(Link { node: to, port })
	})
}

/// Applies the `rule` to the graph nodes. If the rule succeeds,
/// the result is passed to `stitch`, the node is updated, and the old node is returned.
pub const fn single<A, B, S, U>(
	mut rule: A,
	mut stitch: B,
) -> impl FnMut(&mut Nodes<S>, Id) -> Option<Node<S>>
where
	A: FnMut(&mut Nodes<S>, Id) -> Option<U>,
	B: FnMut(&mut Nodes<S>, Id, U) -> Node<S>,
{
	move |nodes, id| {
		rule(nodes, id).map(|result| {
			let node = stitch(nodes, id, result);

			std::mem::replace(&mut nodes[id], node)
		})
	}
}
