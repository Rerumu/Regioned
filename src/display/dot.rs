use std::io::{Result, Write};

use arena::referent::{Referent, Similar};

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
	nodes: Vec<Id>,
	ports: Vec<Ports>,
}

impl Dot {
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			depth_first_searcher: DepthFirstSearcher::new(),
			labels: Vec::new(),
			nodes: Vec::new(),
			ports: Vec::new(),
		}
	}

	fn find_topological<T: Parameters>(&mut self, nodes: &DataFlowGraph<T>, start: Id) {
		self.labels.clear();
		self.nodes.clear();

		self.depth_first_searcher
			.restrict(0..nodes.indices_needed());

		let mut label = Label::A;

		self.depth_first_searcher
			.run(nodes, start, |event| match event {
				Event::PreNode { id } => {
					self.labels.push(label);
					self.nodes.push(id);
				}
				Event::PostNode { .. } => {}
				Event::PreRegion { .. } => label = label.next(),
				Event::PostRegion { .. } => label = label.previous(),
			});
	}

	fn find_ports<T: Parameters>(&mut self, nodes: &DataFlowGraph<T>) {
		self.ports.clear();
		self.ports.resize(nodes.indices_needed(), Ports::new(0, 0));

		for &id in &self.nodes {
			self.ports[id].set_input(nodes[id].parameters().count());

			for &link in nodes[id].parameters() {
				self.ports[link.node].set_output(link.port);
			}

			if let Some(results) = nodes[id].as_results() {
				for &link in results.iter().flatten() {
					self.ports[link.node].set_output(link.port);
				}
			}
		}
	}

	fn write_node_bodies<T>(&self, write: &mut dyn Write, nodes: &DataFlowGraph<T>) -> Result<()>
	where
		T: Description,
	{
		let mut last_label = Label::H;

		for (&id, &label) in self.nodes.iter().zip(&self.labels) {
			if last_label != label {
				last_label = label;

				writeln!(
					write,
					"\tnode [fillcolor = \"{}\", group = {}]",
					label.color(),
					label.name()
				)?;
			}

			write!(write, "\t{id} ")?;

			write_contents(write, &nodes[id], self.ports[id])?;

			if let Some(results) = nodes[id].as_results() {
				for (index, result) in results.iter().enumerate() {
					let ports = Ports::new(result.len().try_into().unwrap(), 0);

					write!(write, "\tR{index}_{id} ")?;

					write_contents(write, &index, ports)?;
				}
			}
		}

		Ok(())
	}

	fn write_node_links<T>(&self, write: &mut dyn Write, nodes: &DataFlowGraph<T>) -> Result<()>
	where
		T: Parameters,
	{
		for &id in &self.nodes {
			if let Some(results) = nodes[id].as_results() {
				for (index_1, result) in results.iter().enumerate() {
					writeln!(write, "\tR{index_1}_{id} -> {id}:e;")?;

					for (index_2, Link { node, port }) in result.iter().enumerate() {
						writeln!(write, "\t{node}:O{port} -> R{index_1}_{id}:I{index_2};")?;
					}
				}
			}

			for (index, Link { node, port }) in nodes[id].parameters().enumerate() {
				writeln!(write, "\t{node}:O{port}:s -> {id}:I{index}:n;")?;
			}
		}

		Ok(())
	}

	fn write_node_outliers<T>(&self, write: &mut dyn Write, nodes: &DataFlowGraph<T>) -> Result<()>
	where
		T: Parameters + Description,
	{
		writeln!(write, "\tsubgraph outliers {{")?;
		writeln!(write, "\t\tnode [fillcolor = \"#000000\"];")?;

		let set = self.depth_first_searcher.set();

		for (id, node) in nodes.iter() {
			if !set.contains(id.index().try_into_unchecked()) {
				continue;
			}

			write!(write, "\t\t{id} ")?;

			write_contents(write, node, Ports::new(0, 0))?;

			for Link { node, .. } in node.parameters() {
				writeln!(write, "\t\t{node} -> {id};")?;
			}
		}

		writeln!(write, "\t}}")
	}

	/// Writes the data flow graph to the writer in the DOT format.
	///
	/// # Errors
	///
	/// Returns an error if the writer fails to write.
	pub fn write<T>(
		&mut self,
		write: &mut dyn Write,
		nodes: &DataFlowGraph<T>,
		start: Id,
	) -> Result<()>
	where
		T: Parameters + Description,
	{
		writeln!(write, "digraph {{")?;
		writeln!(
			write,
			"\tnode [shape = plain, style = filled, ordering = out, color = \"#FFFFFF\", fontcolor = \"#FFFFFF\"];"
		)?;

		self.find_topological(nodes, start);
		self.find_ports(nodes);
		self.write_node_bodies(write, nodes)?;
		self.write_node_links(write, nodes)?;
		self.write_node_outliers(write, nodes)?;

		writeln!(write, "}}")
	}
}

impl Default for Dot {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}
