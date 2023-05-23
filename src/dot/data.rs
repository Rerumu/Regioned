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

fn region_of<S>(node: &Node<S>) -> Option<Region> {
	node.as_compound().and_then(|compound| match *compound {
		Compound::Gamma { .. } => None,
		Compound::Theta { region, .. }
		| Compound::Lambda { region, .. }
		| Compound::Phi { region, .. } => Some(region),
	})
}

fn write_simple<S>(
	ports: &[PortCounts],
	w: &mut dyn Write,
	nodes: &Nodes<S>,
	id: Id,
	place: Id,
) -> Result<()>
where
	S: Parameters + Description,
{
	write!(w, "{id} ")?;
	ports[id].write(w, &nodes[id])?;
	nodes.write_links_in_place(w, id, place)
}

fn write_marker_start(compounds: &HashMap<Id, Id>, w: &mut dyn Write, id: Id) -> Result<()> {
	if let Some(&parent) = compounds.get(&id) {
		writeln!(w, "subgraph cluster_{parent} {{")?;
	}

	writeln!(w, "subgraph cluster_{id} {{")
}

fn write_marker_end(compounds: &HashMap<Id, Id>, w: &mut dyn Write, id: Id) -> Result<()> {
	writeln!(w, "}}")?;

	if let Some(parent) = compounds.get(&id) {
		writeln!(w, "}} // {parent}")?;
	}

	Ok(())
}

fn write_gamma<S>(
	ports: &[PortCounts],
	w: &mut dyn Write,
	nodes: &Nodes<S>,
	regions: &[Region],
) -> Result<()>
where
	S: Parameters + Description,
{
	for (i, region) in regions.iter().enumerate() {
		let start = region.start();
		let end = region.end();

		writeln!(w, "subgraph cluster_{start} {{")?;
		writeln!(w, r#"label = "{i}";"#)?;

		write_simple(ports, w, nodes, start, start)?;
		write_simple(ports, w, nodes, end, end)?;

		writeln!(w, "}}")?;
	}

	Ok(())
}

fn write_compound<S>(
	ports: &[PortCounts],
	w: &mut dyn Write,
	nodes: &Nodes<S>,
	id: Id,
	compound: &Compound,
) -> Result<()>
where
	S: Parameters + Description,
{
	writeln!(w, "subgraph cluster_{id} {{")?;

	Group::from(compound).write(w)?;

	match compound {
		Compound::Gamma { regions, .. } => {
			write_simple(ports, w, nodes, id, id)?;
			write_gamma(ports, w, nodes, regions)?;
		}
		Compound::Theta { region, .. }
		| Compound::Lambda { region, .. }
		| Compound::Phi { region, .. } => {
			write_simple(ports, w, nodes, region.start(), id)?;
			write_simple(ports, w, nodes, region.end(), region.end())?;
		}
	}

	writeln!(w, "}}")
}

fn write_insiders<S, I>(
	topological: &mut ReverseTopological,
	ports: &[PortCounts],
	compounds: &HashMap<Id, Id>,
	w: &mut dyn Write,
	nodes: &Nodes<S>,
	roots: I,
) -> Result<()>
where
	S: Parameters + Description,
	I: IntoIterator<Item = Id>,
{
	for id in topological.iter(nodes, roots) {
		match &nodes[id] {
			Node::Simple(..) => write_simple(ports, w, nodes, id, id)?,
			Node::Marker(Marker::Start) => write_marker_start(compounds, w, id)?,
			Node::Marker(Marker::End { .. }) => write_marker_end(compounds, w, id)?,
			Node::Compound(compound) => write_compound(ports, w, nodes, id, compound)?,
		}
	}

	Ok(())
}

fn write_outsiders<S>(
	topological: &ReverseTopological,
	ports: &[PortCounts],
	w: &mut dyn Write,
	nodes: &Nodes<S>,
) -> Result<()>
where
	S: Parameters + Description,
{
	let seen = topological.seen();

	nodes
		.keys()
		.filter(|&id| !seen[id])
		.try_for_each(|id| write_simple(ports, w, nodes, id, id))
}

trait Extension<S> {
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

impl<S: Parameters> Extension<S> for Nodes<S> {
	fn region_start_of(&self, id: Id) -> Id {
		region_of(&self[id]).map_or(id, Region::start)
	}

	fn region_end_of(&self, id: Id) -> Id {
		region_of(&self[id]).map_or(id, Region::end)
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

	fn initialize<S: Parameters>(&mut self, nodes: &Nodes<S>) {
		self.ports.clear();
		self.ports.resize_with(nodes.active(), Default::default);

		self.compounds.clear();

		for (id, node) in nodes.iter() {
			let start = nodes.region_start_of(id);

			self.ports[start].set_inward(node.parameters().len());

			for parameter in node.parameters() {
				let end = nodes.region_end_of(parameter.node);
				let len = usize::from(parameter.port + 1);

				self.ports[end].set_outward(len);
			}

			if let Node::Compound(compound) = node {
				let regions = compound.regions();

				self.compounds.insert(regions.first().unwrap().start(), id);
				self.compounds.insert(regions.last().unwrap().end(), id);
			}
		}
	}

	/// # Errors
	///
	/// Returns an error if the writer fails to write.
	pub fn write<S, I>(&mut self, writer: &mut dyn Write, nodes: &Nodes<S>, roots: I) -> Result<()>
	where
		S: Parameters + Description,
		I: IntoIterator<Item = Id>,
	{
		const NODE_ATTRIBUTES: &str = r##"shape = plain, style = filled, fillcolor = "#DDDDFF""##;

		writeln!(writer, "digraph {{")?;
		writeln!(writer, "node [{NODE_ATTRIBUTES}];")?;
		writeln!(writer, "style = filled;")?;

		self.initialize(nodes);
		write_insiders(
			&mut self.topological,
			&self.ports,
			&self.compounds,
			writer,
			nodes,
			roots,
		)?;
		write_outsiders(&self.topological, &self.ports, writer, nodes)?;

		writeln!(writer, "}}")
	}
}
