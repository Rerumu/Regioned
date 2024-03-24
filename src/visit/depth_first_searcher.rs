use set::Set;

use crate::collection::{
	data_flow_graph::DataFlowGraph,
	link::{Id, Link},
	node::Parameters,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
	PreRegion { region: u16 },
	PostRegion { region: u16 },
	PreRegions { id: Id },
	PostRegions { id: Id },
	PostParameters { id: Id },
}

pub struct DepthFirstSearcher {
	events: Vec<Event>,
	active: Set,

	parameter_count: Vec<u16>,
	parameter_buffer: Vec<Id>,
}

impl DepthFirstSearcher {
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			events: Vec::new(),
			active: Set::new(),

			parameter_count: Vec::new(),
			parameter_buffer: Vec::new(),
		}
	}

	fn store_parameters<I: Iterator<Item = Id>>(&mut self, iterator: I) {
		let start = self.parameter_buffer.len();

		self.parameter_buffer.extend(iterator);
		self.parameter_buffer[start..].reverse();

		let len = self.parameter_buffer.len() - start;

		self.parameter_count.push(len.try_into().unwrap());
	}

	fn queue_regions(&mut self, id: Id, regions: &[Vec<Link>]) {
		self.events.push(Event::PostRegions { id });

		for (region, list) in regions.iter().enumerate().rev() {
			let region = region.try_into().unwrap();

			self.events.push(Event::PostRegion { region });
			self.events.push(Event::PreRegion { region });

			self.store_parameters(list.iter().map(|link| link.0));
		}

		self.events.push(Event::PreRegions { id });
	}

	fn queue_parameters<T: Parameters>(&mut self, id: Id, data_flow_graph: &DataFlowGraph<T>) {
		self.events.push(Event::PostParameters { id });

		self.store_parameters(data_flow_graph[id].parameters().map(|link| link.0));
	}

	fn queue_node<T: Parameters>(&mut self, id: Id, data_flow_graph: &DataFlowGraph<T>) {
		if !self.active.remove(id.into_usize()).unwrap_or(false) {
			return;
		}

		if let Some(regions) = data_flow_graph[id].as_results() {
			self.queue_regions(id, regions);
		}

		self.queue_parameters(id, data_flow_graph);
	}

	fn fetch_next_id(&mut self) -> Option<Id> {
		let count = self.parameter_count.pop().unwrap();

		count.checked_sub(1).map(|count| {
			self.parameter_count.push(count);
			self.parameter_buffer.pop().unwrap()
		})
	}

	fn fetch_and_queue<T: Parameters>(
		&mut self,
		event: Event,
		data_flow_graph: &DataFlowGraph<T>,
	) -> bool {
		matches!(
			event,
			Event::PostRegion { .. } | Event::PostParameters { .. }
		) && self.fetch_next_id().map_or(false, |id| {
			self.queue_node(id, data_flow_graph);

			true
		})
	}

	#[must_use]
	pub const fn active(&self) -> &Set {
		&self.active
	}

	#[must_use]
	pub fn active_mut(&mut self) -> &mut Set {
		&mut self.active
	}

	pub fn run<T, H>(&mut self, start: Id, data_flow_graph: &DataFlowGraph<T>, mut handler: H)
	where
		T: Parameters,
		H: FnMut(Event),
	{
		if !self.active.contains(start.into_usize()) {
			return;
		}

		self.queue_node(start, data_flow_graph);

		while let Some(&event) = self.events.last() {
			if self.fetch_and_queue(event, data_flow_graph) {
				continue;
			}

			handler(event);

			self.events.pop();
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
		let mut data_flow_graph = DataFlowGraph::<Simple>::new();
		let mut real = [0; 6];
		let mut result = [0; 6];

		let node_0 = data_flow_graph.add_simple(Simple::Leaf);
		let node_1 = data_flow_graph.add_simple(Simple::Ref(node_0.into()));

		real[node_0] = 3;
		real[node_1] = 4;

		let node_2 = data_flow_graph.add_simple(Simple::Leaf);
		let node_3 = data_flow_graph.add_simple(Simple::Leaf);

		real[node_2] = 5;
		real[node_3] = 6;

		let node_4 = data_flow_graph.add_simple(Simple::Leaf);
		let node_5 = data_flow_graph.add_gamma(
			vec![node_4.into()],
			vec![vec![node_1.into()], vec![node_2.into(), node_3.into()]],
		);

		real[node_4] = 1;
		real[node_5] = 2;

		let mut searcher = DepthFirstSearcher::new();
		let mut counter = 0;

		let active = searcher.active_mut();

		active.clear();
		active.extend(0..data_flow_graph.nodes().len());

		searcher.run(node_5, &data_flow_graph, |event| {
			if let Event::PostParameters { id } = event {
				println!("POST {id:?}");

				counter += 1;

				result[id] = counter;
			}
		});

		assert_eq!(result, real);
	}
}
