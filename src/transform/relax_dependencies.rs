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

	fn add_map_results<N>(&mut self, nodes: &Nodes<N>, parameters: &[Link], region: Region)
	where
		N: Parameters,
	{
		let iter = nodes[region.end]
			.parameters()
			.map(|&end| load_passthrough(parameters, region.start, end));

		self.maps.clear();
		self.maps.extend(iter);
	}

	fn run_gamma<N>(&mut self, nodes: &Nodes<N>, parameters: &[Link], regions: &[Region])
	where
		N: Parameters,
	{
		let mut regions = regions.iter();
		let region = *regions.next().expect("`Gamma` has no region");

		self.add_map_results(nodes, parameters, region);

		for &Region { start, end } in regions {
			let results = nodes[end].parameters();

			for (&result, old) in results.zip(&mut self.maps) {
				if load_passthrough(parameters, start, result) != *old {
					*old = None;
				}
			}
		}
	}

	fn run_theta<N>(&mut self, nodes: &Nodes<N>, parameters: &[Link], region: Region)
	where
		N: Parameters,
	{
		// This will technically include the `Theta` condition, but
		// it won't be used anyway as it's not output.
		self.add_map_results(nodes, parameters, region);

		let results = nodes[region.end].parameters();
		let inputs = Link::from(region.start).iter();

		for ((&result, start), old) in results.zip(inputs).zip(&mut self.maps) {
			if result != start {
				*old = None;
			}
		}
	}

	pub fn run<N>(&mut self, nodes: &mut Nodes<N>, id: Id, successors: &Successors) -> Option<usize>
	where
		N: Parameters + ParametersMut,
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
