use std::ops::Range;

use arena::referent::{Referent, Similar};
use set::Set;

use crate::collection::{
	data_flow_graph::DataFlowGraph,
	link::{Id, Link},
	node::Parameters,
};

fn store_iterator<I: Iterator<Item = Id>>(data: &mut Vec<Id>, iterator: I) -> Range<usize> {
	let start = data.len();

	data.extend(iterator);
	data[start..].reverse();

	start..data.len()
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
	PreNode { id: Id },
	PostNode { id: Id },
	PreRegion { id: Id, region: usize },
	PostRegion { id: Id, region: usize },
}

struct Item {
	event: Event,
	parameters: Range<usize>,
}

pub struct DepthFirstSearcher {
	items: Vec<Item>,
	set: Set,

	parameters: Vec<Id>,
}

impl DepthFirstSearcher {
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			items: Vec::new(),
			set: Set::new(),

			parameters: Vec::new(),
		}
	}

	fn queue_pre_node(&mut self, id: Id) {
		let parameters = self.parameters.len()..self.parameters.len();

		self.items.push(Item {
			event: Event::PreNode { id },
			parameters,
		});
	}

	fn queue_post_node<T: Parameters>(&mut self, nodes: &DataFlowGraph<T>, id: Id) {
		let parameters = store_iterator(
			&mut self.parameters,
			nodes[id].parameters().map(|link| link.node),
		);

		self.items.push(Item {
			event: Event::PostNode { id },
			parameters,
		});
	}

	fn queue_pre_region(&mut self, id: Id, region: usize) {
		let parameters = self.parameters.len()..self.parameters.len();

		self.items.push(Item {
			event: Event::PreRegion { id, region },
			parameters,
		});
	}

	fn queue_post_region(&mut self, id: Id, region: usize, list: &[Link]) {
		let parameters = store_iterator(&mut self.parameters, list.iter().map(|link| link.node));

		self.items.push(Item {
			event: Event::PostRegion { id, region },
			parameters,
		});
	}

	fn queue_node<T: Parameters>(&mut self, nodes: &DataFlowGraph<T>, id: Id) {
		if !self.set.remove(id.index().try_into_unchecked()) {
			return;
		}

		let regions = nodes[id].as_results().unwrap_or_default();

		for (region, list) in regions.iter().enumerate().rev() {
			self.queue_post_region(id, region, list);
			self.queue_pre_region(id, region);
		}

		self.queue_post_node(nodes, id);
		self.queue_pre_node(id);
	}

	#[inline]
	#[must_use]
	pub const fn set(&self) -> &Set {
		&self.set
	}

	pub fn restrict<I: IntoIterator<Item = usize>>(&mut self, set: I) {
		self.set.clear();
		self.set.extend(set);
	}

	pub fn run<T, H>(&mut self, nodes: &DataFlowGraph<T>, start: Id, mut handler: H)
	where
		T: Parameters,
		H: FnMut(Event),
	{
		self.queue_node(nodes, start);

		while let Some(mut item) = self.items.pop() {
			if let Some(parameter) = item.parameters.next_back() {
				self.items.push(item);

				self.queue_node(nodes, self.parameters[parameter]);
			} else {
				handler(item.event);

				self.parameters.truncate(item.parameters.start);
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

	use super::{DepthFirstSearcher, Event};

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
		let mut real = [0; 6];
		let mut result = [0; 6];

		let node_0 = nodes.add_simple(Simple::Leaf);
		let node_1 = nodes.add_simple(Simple::Ref(node_0));

		real[node_0.node] = 3;
		real[node_1.node] = 4;

		let node_2 = nodes.add_simple(Simple::Leaf);
		let node_3 = nodes.add_simple(Simple::Leaf);

		real[node_2.node] = 5;
		real[node_3.node] = 6;

		let node_4 = nodes.add_simple(Simple::Leaf);
		let node_5 = nodes.add_gamma(vec![node_4], tiny_vec![vec![node_1], vec![node_2, node_3]]);

		real[node_4.node] = 1;
		real[node_5.node] = 2;

		let mut searcher = DepthFirstSearcher::new();
		let mut counter = 0;

		searcher.restrict(0..nodes.indices_needed());
		searcher.run(&nodes, node_5.node, |event| {
			if let Event::PostNode { id } = event {
				println!("POST {id}");

				counter += 1;

				result[id] = counter;
			}
		});

		assert_eq!(result, real);
	}
}
