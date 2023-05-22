use crate::{
	data_flow::{
		link::{Id, Link, Region},
		node::{Compound, Parameters, ParametersMut},
		nodes::Nodes,
	},
	transform::revise::redo_ports,
	visit::successors::Successors,
};

fn load_passthrough(parameters: &[Link], start: Id, end: Link) -> Option<Link> {
	if start == end.node {
		let port = usize::from(end.port);

		Some(parameters[port])
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

	fn add_map_results<S>(&mut self, nodes: &Nodes<S>, parameters: &[Link], region: Region)
	where
		S: Parameters,
	{
		let iter = nodes[region.end()]
			.parameters()
			.map(|&end| load_passthrough(parameters, region.start(), end));

		self.maps.clear();
		self.maps.extend(iter);
	}

	fn run_gamma<S>(&mut self, nodes: &Nodes<S>, parameters: &[Link], regions: &[Region])
	where
		S: Parameters,
	{
		let mut regions = regions.iter();
		let region = *regions.next().expect("`Gamma` has no region");

		self.add_map_results(nodes, parameters, region);

		for region in regions {
			let results = nodes[region.end()].parameters();

			for (&result, old) in results.zip(&mut self.maps) {
				if load_passthrough(parameters, region.start(), result) != *old {
					*old = None;
				}
			}
		}
	}

	fn run_theta<S>(&mut self, nodes: &Nodes<S>, parameters: &[Link], region: Region)
	where
		S: Parameters,
	{
		// This will technically include the `Theta` condition, but
		// it won't be used anyway as it's not output.
		self.add_map_results(nodes, parameters, region);

		let results = nodes[region.end()].parameters();
		let inputs = Link::from(region.start()).iter();

		for ((&result, start), old) in results.zip(inputs).zip(&mut self.maps) {
			if result != start {
				*old = None;
			}
		}
	}

	pub fn run<S>(&mut self, nodes: &mut Nodes<S>, id: Id, successors: &Successors) -> Option<usize>
	where
		S: Parameters + ParametersMut,
	{
		let compound = nodes[id].as_compound()?;

		match &compound {
			Compound::Gamma {
				parameters,
				regions,
			} => self.run_gamma(nodes, parameters, regions),
			Compound::Theta { parameters, region } => self.run_theta(nodes, parameters, *region),
			_ => return None,
		}

		let result = redo_ports(nodes, successors, id, |port| self.maps[usize::from(port)]);

		Some(result)
	}
}
