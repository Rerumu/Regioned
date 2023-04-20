use crate::{
	data_flow::{
		graph::{Graph, PredecessorList},
		link::Link,
		node::{Compound, Id, Node, Region},
	},
	transform::revise::redo_ports,
	visit::successors::Successors,
};

fn load_passthrough(predecessors: &PredecessorList, link: Link, start: Id) -> Option<Link> {
	if link.node() == start {
		let port = usize::from(link.port());
		let previous = predecessors[port];

		Some(previous)
	} else {
		None
	}
}

#[derive(Default)]
pub struct RelaxDependencies {
	maps: Vec<Option<Link>>,
}

impl RelaxDependencies {
	#[must_use]
	pub const fn new() -> Self {
		Self { maps: Vec::new() }
	}

	fn add_map_results(&mut self, predecessors: &[PredecessorList], region: Region, parent: Id) {
		let iter = predecessors[region.end()]
			.iter()
			.map(|&link| load_passthrough(&predecessors[parent], link, region.start()));

		self.maps.clear();
		self.maps.extend(iter);
	}

	fn run_gamma<S>(&mut self, graph: &Graph<S>, id: Id) {
		let mut regions = graph.regions[&id].iter();
		let region = *regions.next().expect("`Gamma` has no region");

		self.add_map_results(&graph.predecessors, region, id);

		for region in regions {
			let results = graph.predecessors[region.end()].iter().copied();

			for (link, old) in results.zip(&mut self.maps) {
				if load_passthrough(&graph.predecessors[id], link, region.start()) != *old {
					*old = None;
				}
			}
		}
	}

	fn run_theta<S>(&mut self, graph: &Graph<S>, id: Id) {
		let region = *graph.regions[&id].first().expect("`Theta` has no region");

		// This will technically include the `Theta` condition, but
		// it won't be used anyway as it's not output.
		self.add_map_results(&graph.predecessors, region, id);

		let results = graph.predecessors[region.end()].iter().copied();
		let inputs = Link::from(region.start()).iter();

		for ((result, start), old) in results.zip(inputs).zip(&mut self.maps) {
			if result != start {
				*old = None;
			}
		}
	}

	pub fn run<S>(&mut self, graph: &mut Graph<S>, id: Id, successors: &Successors) -> usize {
		match graph.nodes[id] {
			Node::Compound(Compound::Gamma) => self.run_gamma(graph, id),
			Node::Compound(Compound::Theta) => self.run_theta(graph, id),
			_ => return 0,
		}

		redo_ports(&mut graph.predecessors, successors, id, |port| {
			self.maps[usize::from(port)]
		})
	}
}
