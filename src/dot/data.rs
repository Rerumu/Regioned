use std::{
	collections::HashMap,
	io::{Result, Write},
};

use crate::{
	data_flow::{
		link::{Id, Link, Region},
		node::{Compound, Marker, Node, Parameters},
		nodes::Nodes,
	},
	visit::reverse_topological::ReverseTopological,
};

use super::{
	description::Description,
	template::{Anchor, Group, PortCounts},
};

fn region_of<N>(node: &Node<N>) -> Option<Region> {
	node.as_compound().and_then(|compound| match *compound {
		Compound::Gamma { .. } => None,
		Compound::Theta { region, .. }
		| Compound::Lambda { region, .. }
		| Compound::Phi { region, .. } => Some(region),
	})
}

trait Extension<N> {
	fn region_start_of(&self, id: Id) -> Id;

	fn region_end_of(&self, id: Id) -> Id;

	fn write_link_between(&self, w: &mut dyn Write, from: Link, to: Link) -> Result<()> {
		Anchor::Out.write(w, self.region_end_of(from.node), from.port)?;
		write!(w, " -> ")?;
		Anchor::In.write(w, self.region_start_of(to.node), to.port)?;
		writeln!(w, ";")
	}

	fn write_links_in_place(&self, w: &mut dyn Write, to: Id, from: Id) -> Result<()>;
}

impl<N: Parameters> Extension<N> for Nodes<N> {
	fn region_start_of(&self, id: Id) -> Id {
		region_of(&self[id]).map_or(id, |region| region.start)
	}

	fn region_end_of(&self, id: Id) -> Id {
		region_of(&self[id]).map_or(id, |region| region.end)
	}

	fn write_links_in_place(&self, w: &mut dyn Write, id: Id, place: Id) -> Result<()> {
		for (i, &from) in self[place].parameters().enumerate() {
			let port = i.try_into().unwrap();
			let to = Link { node: id, port };

			self.write_link_between(w, from, to)?;
		}

		Ok(())
	}
}

#[derive(Default)]
pub struct Dot {
	topological: ReverseTopological,
	ports: Vec<PortCounts>,
	compounds: HashMap<Id, Id>,
}

impl Dot {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn initialize<N: Parameters>(&mut self, nodes: &Nodes<N>) {
		self.ports.clear();
		self.ports.resize_with(nodes.active(), Default::default);

		self.compounds.clear();

		for (id, node) in nodes.iter() {
			let start = nodes.region_start_of(id);

			self.ports[start].set_inward(node.parameters().count());

			for parameter in node.parameters() {
				let end = nodes.region_end_of(parameter.node);
				let len = usize::from(parameter.port + 1);

				self.ports[end].set_outward(len);
			}

			if let Node::Compound(compound) = node {
				let regions = compound.regions();

				self.compounds.insert(regions.first().unwrap().start, id);
				self.compounds.insert(regions.last().unwrap().end, id);
			}
		}
	}

	fn write_simple<N>(&self, w: &mut dyn Write, nodes: &Nodes<N>, id: Id, place: Id) -> Result<()>
	where
		N: Parameters + Description,
	{
		write!(w, "{id} ")?;
		self.ports[id].write(w, &nodes[id])?;
		nodes.write_links_in_place(w, id, place)
	}

	fn write_marker_start(&self, w: &mut dyn Write, id: Id) -> Result<()> {
		if let Some(&parent) = self.compounds.get(&id) {
			writeln!(w, "subgraph cluster_{parent} {{")?;
		}

		writeln!(w, "subgraph cluster_{id} {{")
	}

	fn write_marker_end(&self, w: &mut dyn Write, id: Id) -> Result<()> {
		writeln!(w, "}}")?;

		if let Some(parent) = self.compounds.get(&id) {
			writeln!(w, "}} // {parent}")?;
		}

		Ok(())
	}

	fn write_gamma<N>(&self, w: &mut dyn Write, nodes: &Nodes<N>, regions: &[Region]) -> Result<()>
	where
		N: Parameters + Description,
	{
		for (i, &Region { start, end }) in regions.iter().enumerate() {
			writeln!(w, "subgraph cluster_{start} {{")?;
			writeln!(w, r#"label = "{i}";"#)?;

			self.write_simple(w, nodes, start, start)?;
			self.write_simple(w, nodes, end, end)?;

			writeln!(w, "}}")?;
		}

		Ok(())
	}

	fn write_compound<N>(
		&self,
		w: &mut dyn Write,
		nodes: &Nodes<N>,
		id: Id,
		compound: &Compound,
	) -> Result<()>
	where
		N: Parameters + Description,
	{
		writeln!(w, "subgraph cluster_{id} {{")?;

		Group::from(compound).write(w)?;

		match compound {
			Compound::Gamma { regions, .. } => {
				self.write_simple(w, nodes, id, id)?;
				self.write_gamma(w, nodes, regions)?;
			}
			Compound::Theta { region, .. }
			| Compound::Lambda { region, .. }
			| Compound::Phi { region, .. } => {
				self.write_simple(w, nodes, region.start, id)?;
				self.write_simple(w, nodes, region.end, region.end)?;
			}
		}

		writeln!(w, "}}")
	}

	fn write_insiders<N, I>(&mut self, w: &mut dyn Write, nodes: &Nodes<N>, roots: I) -> Result<()>
	where
		N: Parameters + Description,
		I: IntoIterator<Item = Id>,
	{
		let mut topological = std::mem::take(&mut self.topological);

		for id in topological.iter(nodes, roots) {
			match &nodes[id] {
				Node::Simple(..) => self.write_simple(w, nodes, id, id)?,
				Node::Marker(Marker::Start) => self.write_marker_start(w, id)?,
				Node::Marker(Marker::End { .. }) => self.write_marker_end(w, id)?,
				Node::Compound(compound) => self.write_compound(w, nodes, id, compound)?,
			}
		}

		self.topological = topological;

		Ok(())
	}

	fn write_outsiders<N>(&self, w: &mut dyn Write, nodes: &Nodes<N>) -> Result<()>
	where
		N: Parameters + Description,
	{
		let seen = self.topological.seen();

		nodes
			.keys()
			.filter(|&id| !seen[id])
			.try_for_each(|id| self.write_simple(w, nodes, id, id))
	}

	/// # Errors
	///
	/// Returns an error if the writer fails to write.
	pub fn write<N, I>(&mut self, writer: &mut dyn Write, nodes: &Nodes<N>, roots: I) -> Result<()>
	where
		N: Parameters + Description,
		I: IntoIterator<Item = Id>,
	{
		const NODE_ATTRIBUTES: &str = r##"shape = plain, style = filled, fillcolor = "#DDDDFF""##;

		writeln!(writer, "digraph {{")?;
		writeln!(writer, "node [{NODE_ATTRIBUTES}];")?;
		writeln!(writer, "style = filled;")?;

		self.initialize(nodes);
		self.write_insiders(writer, nodes, roots)?;
		self.write_outsiders(writer, nodes)?;

		writeln!(writer, "}}")
	}
}
