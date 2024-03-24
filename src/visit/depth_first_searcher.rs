use arena::referent::{Referent, Similar};
use set::Set;

use crate::collection::{data_flow_graph::DataFlowGraph, link::Id, node::Parameters};

struct Item {
	id: Id,
	parameters: Vec<Id>,
}

pub struct DepthFirstSearcher {
	items: Vec<Item>,
	unseen: Set,

	vec_pooled: Vec<Vec<Id>>,
}

impl DepthFirstSearcher {
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			items: Vec::new(),
			unseen: Set::new(),
			vec_pooled: Vec::new(),
		}
	}

	fn queue_item<T, H>(&mut self, nodes: &DataFlowGraph<T>, id: Id, mut handler: H)
	where
		T: Parameters,
		H: FnMut(Id, bool),
	{
		if !self.unseen.remove(id.index().try_into_unchecked()) {
			return;
		}

		let mut parameters = self.vec_pooled.pop().unwrap_or_default();
		let node = &nodes[id];

		if let Some(results) = node.as_results() {
			parameters.extend(results.iter().flatten().map(|link| link.node));
		}

		parameters.extend(node.parameters().map(|link| link.node));
		parameters.reverse();

		self.items.push(Item { id, parameters });

		handler(id, false);
	}

	#[inline]
	#[must_use]
	pub const fn unseen(&self) -> &Set {
		&self.unseen
	}

	pub fn restrict<I: IntoIterator<Item = usize>>(&mut self, set: I) {
		self.unseen.clear();
		self.unseen.extend(set);
	}

	pub fn run<T, H>(&mut self, nodes: &DataFlowGraph<T>, start: Id, mut handler: H)
	where
		T: Parameters,
		H: FnMut(Id, bool),
	{
		self.queue_item(nodes, start, &mut handler);

		while let Some(mut item) = self.items.pop() {
			if let Some(parameter) = item.parameters.pop() {
				self.items.push(item);

				self.queue_item(nodes, parameter, &mut handler);
			} else {
				handler(item.id, true);

				self.vec_pooled.push(item.parameters);
			}
		}
	}
}

impl Default for DepthFirstSearcher {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use tinyvec::tiny_vec;

	use crate::collection::{data_flow_graph::DataFlowGraph, link::Link, node::Parameters};

	use super::DepthFirstSearcher;

	enum Simple {
		Leaf,
		Ref(Link),
	}

	impl Parameters for Simple {
		type Iter<'a> = std::option::IntoIter<&'a Link>;

		fn parameters(&self) -> Self::Iter<'_> {
			let parameters = match self {
				Self::Leaf => None,
				Self::Ref(link) => Some(link),
			};

			parameters.into_iter()
		}
	}

	#[test]
	fn test_is_in_order() {
		let mut nodes = DataFlowGraph::<Simple>::new();
		let mut expected = [0; 6];

		let value_1 = nodes.add_simple(Simple::Leaf);
		let value_2 = nodes.add_simple(Simple::Ref(value_1));

		expected[value_1.node] = 1;
		expected[value_2.node] = 2;

		let value_3 = nodes.add_simple(Simple::Leaf);
		let value_4 = nodes.add_simple(Simple::Leaf);

		expected[value_3.node] = 3;
		expected[value_4.node] = 4;

		let value_5 = nodes.add_simple(Simple::Leaf);
		let gamma = nodes.add_gamma(
			vec![value_5],
			tiny_vec![vec![value_2], vec![value_3, value_4]],
		);

		expected[value_5.node] = 5;
		expected[gamma.node] = 6;

		let mut searcher = DepthFirstSearcher::new();
		let mut counter = 0;

		searcher.restrict(0..nodes.indices_needed());
		searcher.run(&nodes, gamma.node, |id, post| {
			if !post {
				return;
			}

			counter += 1;

			assert_eq!(expected[id], counter, "Node {id} was not in order");
		});

		assert_eq!(counter, 6, "Not all nodes were visited");
	}
}
