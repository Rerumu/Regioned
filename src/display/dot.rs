use std::io::{Result, Write};

use crate::{
	collection::{
		data_flow_graph::DataFlowGraph,
		link::{Id, Link},
		node::Parameters,
	},
	visit::depth_first_searcher::{DepthFirstSearcher, Event},
};

use super::{
	description::Description,
	node::{write_contents, Ports},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Label {
	A,
	B,
	C,
	D,
	E,
	F,
	G,
	H,
}

impl Label {
	const fn previous(self) -> Self {
		match self {
			Self::A => Self::H,
			Self::B => Self::A,
			Self::C => Self::B,
			Self::D => Self::C,
			Self::E => Self::D,
			Self::F => Self::E,
			Self::G => Self::F,
			Self::H => Self::G,
		}
	}

	const fn next(self) -> Self {
		match self {
			Self::A => Self::B,
			Self::B => Self::C,
			Self::C => Self::D,
			Self::D => Self::E,
			Self::E => Self::F,
			Self::F => Self::G,
			Self::G => Self::H,
			Self::H => Self::A,
		}
	}

	const fn color(self) -> &'static str {
		match self {
			Self::A => "#1ABC9D",
			Self::B => "#2FCC71",
			Self::C => "#3498DB",
			Self::D => "#9B59B6",
			Self::E => "#E91E63",
			Self::F => "#F1C40F",
			Self::G => "#E67E23",
			Self::H => "#E74B3C",
		}
	}

	const fn name(self) -> char {
		match self {
			Self::A => 'A',
			Self::B => 'B',
			Self::C => 'C',
			Self::D => 'D',
			Self::E => 'E',
			Self::F => 'F',
			Self::G => 'G',
			Self::H => 'H',
		}
	}
}

pub struct Dot {
	depth_first_searcher: DepthFirstSearcher,
	labels: Vec<Label>,
	ids: Vec<Id>,
	ports: Vec<Ports>,
}

impl Dot {
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			depth_first_searcher: DepthFirstSearcher::new(),
			labels: Vec::new(),
			ids: Vec::new(),
			ports: Vec::new(),
		}
	}

	fn find_topological<T, I>(&mut self, results: I, data_flow_graph: &DataFlowGraph<T>)
	where
		T: Parameters,
		I: IntoIterator<Item = Id>,
	{
		let active = self.depth_first_searcher.active_mut();

		active.clear();
		active.extend(0..data_flow_graph.len());

		self.labels.clear();
		self.ids.clear();

		for id in results {
			let mut label = Label::A;

			self.depth_first_searcher
				.run(id, data_flow_graph, |event| match event {
					Event::PreRegion { .. } | Event::PostRegion { .. } => {}
					Event::PreRegions { .. } => label = label.next(),
					Event::PostRegions { .. } => label = label.previous(),
					Event::PostParameters { id } => {
						self.labels.push(label);
						self.ids.push(id);
					}
				});
		}
	}

	fn find_ports<T: Parameters>(&mut self, data_flow_graph: &DataFlowGraph<T>) {
		self.ports.clear();
		self.ports.resize(data_flow_graph.len(), Ports::new(0, 0));

		for &id in &self.ids {
			self.ports[id].set_input(data_flow_graph[id].parameters().count());

			for &Link(id, port) in data_flow_graph[id].parameters() {
				self.ports[id].set_output(port);
			}

			if let Some(results) = data_flow_graph[id].as_results() {
				for &Link(id, port) in results.iter().flatten() {
					self.ports[id].set_output(port);
				}
			}
		}
	}

	fn write_node_bodies<T>(
		&self,
		write: &mut dyn Write,
		data_flow_graph: &DataFlowGraph<T>,
	) -> Result<()>
	where
		T: Description,
	{
		let mut last_label = Label::H;

		for (&id, &label) in self.ids.iter().zip(&self.labels) {
			if last_label != label {
				last_label = label;

				writeln!(
					write,
					"\tnode [fillcolor = \"{}\", group = {}]",
					label.color(),
					label.name()
				)?;
			}

			let id_usize = id.into_usize();

			write!(write, "\tN{id_usize} ")?;

			write_contents(write, &data_flow_graph[id], self.ports[id])?;

			if let Some(results) = data_flow_graph[id].as_results() {
				for (index, result) in results.iter().enumerate() {
					let ports = Ports::new(result.len().try_into().unwrap(), 0);

					write!(write, "\tR{index}_{id_usize} ")?;

					write_contents(write, &index, ports)?;
				}
			}
		}

		Ok(())
	}

	fn write_node_links<T>(
		&self,
		write: &mut dyn Write,
		data_flow_graph: &DataFlowGraph<T>,
	) -> Result<()>
	where
		T: Parameters,
	{
		for &successor in &self.ids {
			let successor_usize = successor.into_usize();

			if let Some(results) = data_flow_graph[successor].as_results() {
				for (index_1, region) in results.iter().enumerate() {
					writeln!(
						write,
						"\tR{index_1}_{successor_usize} -> N{successor_usize}:e;"
					)?;

					for (index_2, &Link(id, port)) in region.iter().enumerate() {
						let id_usize = id.into_usize();

						writeln!(
							write,
							"\tN{id_usize}:O{port} -> R{index_1}_{successor_usize}:I{index_2};"
						)?;
					}
				}
			}

			for (index, &Link(id, port)) in data_flow_graph[successor].parameters().enumerate() {
				let id_usize = id.into_usize();

				writeln!(
					write,
					"\tN{id_usize}:O{port}:s -> N{successor_usize}:I{index}:n;"
				)?;
			}
		}

		Ok(())
	}

	fn write_node_outliers<T>(
		&self,
		write: &mut dyn Write,
		data_flow_graph: &DataFlowGraph<T>,
	) -> Result<()>
	where
		T: Parameters + Description,
	{
		writeln!(write, "\tsubgraph outliers {{")?;
		writeln!(write, "\t\tnode [fillcolor = \"#000000\"];")?;

		let active = self.depth_first_searcher.active();

		for (successor, node) in data_flow_graph.iter().enumerate() {
			if !active.contains(successor) {
				continue;
			}

			write!(write, "\t\tN{successor} ")?;

			write_contents(write, node, Ports::new(0, 0))?;

			for &Link(id, ..) in node.parameters() {
				let id_usize = id.into_usize();

				writeln!(write, "\t\tN{id_usize} -> N{successor};")?;
			}
		}

		writeln!(write, "\t}}")
	}

	/// Writes the data flow graph to the writer in the DOT format.
	///
	/// # Errors
	///
	/// Returns an error if the writer fails to write.
	pub fn write<T, I>(
		&mut self,
		write: &mut dyn Write,
		results: I,
		data_flow_graph: &DataFlowGraph<T>,
	) -> Result<()>
	where
		T: Parameters + Description,
		I: IntoIterator<Item = Id>,
	{
		writeln!(write, "digraph {{")?;
		writeln!(
			write,
			"\tnode [shape = plain, style = filled, ordering = out, color = \"#FFFFFF\", fontcolor = \"#FFFFFF\"];"
		)?;

		self.find_topological(results, data_flow_graph);
		self.find_ports(data_flow_graph);
		self.write_node_bodies(write, data_flow_graph)?;
		self.write_node_links(write, data_flow_graph)?;
		self.write_node_outliers(write, data_flow_graph)?;

		writeln!(write, "}}")
	}
}

impl Default for Dot {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}
