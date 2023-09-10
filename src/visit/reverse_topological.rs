use crate::data_flow::{
	link::{Id, Region},
	node::Parameters,
	nodes::Nodes,
};

enum Entry {
	Predecessors { id: Id, index: usize },
	Regions { id: Id, index: usize },
	Node { id: Id },
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

	fn add_node(&mut self, id: Id) {
		if self.seen[id] {
			return;
		}

		self.seen[id] = true;
		self.stack.push(Entry::Predecessors { id, index: 0 });
	}

	fn add_region(&mut self, region: Region) {
		self.add_node(region.end);
		self.add_node(region.start);
	}

	fn handle_predecessor<N: Parameters>(&mut self, nodes: &Nodes<N>, index: usize, id: Id) {
		let node = &nodes[id];
		let index = index + 1;

		if let Some(parameter) = node.parameters().nth(index - 1) {
			self.stack.push(Entry::Predecessors { id, index });

			self.add_node(parameter.node);
		} else if node.as_compound().is_some() {
			self.handle_region(nodes, 0, id);
		} else {
			self.stack.push(Entry::Node { id });
		}
	}

	fn handle_region<N>(&mut self, nodes: &Nodes<N>, index: usize, id: Id) {
		let compound = nodes[id].as_compound().unwrap();
		let index = index + 1;

		if let Some(region) = compound.regions().iter().nth(index - 1).copied() {
			self.stack.push(Entry::Regions { id, index });

			self.add_region(region);
		} else {
			self.stack.push(Entry::Node { id });
		}
	}

	#[inline]
	fn next_in<N: Parameters>(&mut self, nodes: &Nodes<N>) -> Option<Id> {
		loop {
			match self.stack.pop()? {
				Entry::Predecessors { id, index } => self.handle_predecessor(nodes, index, id),
				Entry::Regions { id, index } => self.handle_region(nodes, index, id),
				Entry::Node { id } => return Some(id),
			}
		}
	}

	fn set_up_roots<I>(&mut self, active: usize, roots: I)
	where
		I: IntoIterator<Item = Id>,
	{
		self.seen.clear();
		self.seen.resize(active, false);

		self.stack.clear();

		for id in roots {
			self.add_node(id);
		}

		self.stack.reverse();
	}

	/// Returns an iterator over the nodes in reverse topological order.
	#[inline]
	#[must_use]
	pub fn iter<'a, 'b, N, I>(&'a mut self, nodes: &'b Nodes<N>, roots: I) -> Iter<'a, 'b, N>
	where
		N: Parameters,
		I: IntoIterator<Item = Id>,
	{
		let topological = self;

		topological.set_up_roots(nodes.active(), roots);

		Iter { topological, nodes }
	}
}

/// An iterator over the nodes in reverse topological order.
pub struct Iter<'a, 'b, N> {
	topological: &'a mut ReverseTopological,
	nodes: &'b Nodes<N>,
}

impl<'a, 'b, N: Parameters> Iterator for Iter<'a, 'b, N> {
	type Item = Id;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.topological.next_in(self.nodes)
	}
}

impl<'a, 'b, N: Parameters> std::iter::FusedIterator for Iter<'a, 'b, N> {}

#[cfg(test)]
mod tests {
	use crate::data_flow::{
		link::Link,
		node::{AsParametersMut, Parameters},
		nodes::Nodes,
	};

	use super::ReverseTopological;

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

	impl AsParametersMut for Simple {
		fn as_parameters_mut(&mut self) -> Option<&mut Vec<Link>> {
			None
		}
	}

	#[test]
	fn test_is_in_order() {
		let mut nodes = Nodes::new();

		let region_1 = nodes.add_region();
		let value_1 = nodes.add_simple(Simple::Ref(region_1.start.into()));
		let value_2 = nodes.add_simple(Simple::Ref(value_1.into()));

		nodes[region_1.end]
			.as_parameters_mut()
			.unwrap()
			.push(value_2.into());

		let region_2 = nodes.add_region();
		let value_3 = nodes.add_simple(Simple::Ref(region_2.start.into()));
		let value_4 = nodes.add_simple(Simple::Ref(region_2.start.into()));

		nodes[region_2.end]
			.as_parameters_mut()
			.unwrap()
			.extend([Link::from(value_3), Link::from(value_4)]);

		let value_5 = nodes.add_simple(Simple::Leaf);
		let gamma = nodes.add_gamma([region_1, region_2].into());

		let mut counter = 0;
		let mut expected = vec![0; nodes.active()];

		expected[region_1.start] = 1;
		expected[value_1] = 2;
		expected[value_2] = 3;
		expected[region_1.end] = 4;

		expected[region_2.start] = 5;
		expected[value_3] = 6;
		expected[value_4] = 7;
		expected[region_2.end] = 8;

		expected[gamma] = 9;
		expected[value_5] = 10;

		for id in ReverseTopological::new().iter(&nodes, [gamma, value_5]) {
			counter += 1;

			assert_eq!(expected[id], counter, "Node {id} was not in order");
		}
	}
}
