use std::io::{Result, Write};

use hashbrown::HashMap;

use crate::data_flow::{
	graph::Graph,
	link::Link,
	node::{Compound, Id, Node, Region},
};

use super::{
	label::{Label, Ref},
	node::{Face, Information},
	region::{Labeled, Named},
};

trait Extension<S> {
	fn get_face_region(&self, id: Id) -> Option<Region>;

	fn get_face_incoming(&self, id: Id) -> Id {
		self.get_face_region(id).map_or(id, Region::start)
	}

	fn get_face_outgoing(&self, id: Id) -> Id {
		self.get_face_region(id).map_or(id, Region::end)
	}

	fn add_link_part(&self, w: &mut dyn Write, link: Link, face: Face) -> Result<()> {
		let node = self.get_face_outgoing(link.node());
		let port = link.port().index();
		let side = face.name();
		let direction = face.direction();

		write!(w, "{node}:{side}{port}:{direction}")
	}

	fn add_link(&self, w: &mut dyn Write, from: Link, to: Link) -> Result<()> {
		self.add_link_part(w, from, Face::Outgoing)?;
		write!(w, " -> ")?;
		self.add_link_part(w, to, Face::Incoming)?;
		writeln!(w, ";")
	}

	fn add_links_redirected(&self, w: &mut dyn Write, to: Id, from: Id) -> Result<()>;

	fn add_links_incoming(&self, w: &mut dyn Write, to: Id) -> Result<()> {
		self.add_links_redirected(w, to, to)
	}
}

impl<S> Extension<S> for Graph<S> {
	fn get_face_region(&self, id: Id) -> Option<Region> {
		self.nodes.get(id).and_then(|n| {
			n.as_compound().and_then(|v| match v {
				Compound::Gamma => None,
				_ => self.regions.get(&id).and_then(|v| v.first()).copied(),
			})
		})
	}

	fn add_links_redirected(&self, w: &mut dyn Write, to: Id, from: Id) -> Result<()> {
		self.predecessors[from]
			.iter()
			.copied()
			.enumerate()
			.try_for_each(|(i, from)| {
				let to = Link::new(to, i.try_into().unwrap());

				self.add_link(w, from, to)
			})
	}
}

pub struct Dot<'a, S> {
	information: HashMap<Id, Information>,
	seen: Vec<bool>,
	graph: &'a Graph<S>,
}

impl<'a, S> Dot<'a, S> {
	/// Creates a new [`Dot`].
	#[must_use]
	pub fn new(graph: &'a Graph<S>) -> Self {
		Self {
			information: HashMap::new(),
			seen: Vec::new(),
			graph,
		}
	}

	fn initialize_info(&mut self) {
		self.information.clear();

		for id in self.graph.nodes.keys() {
			let list = &self.graph.predecessors[id];
			let face = self.graph.get_face_incoming(id);

			self.information
				.entry(face)
				.or_default()
				.set_incoming(list.len());

			for link in list {
				let face = self.graph.get_face_outgoing(link.node());
				let value = usize::from(link.port()) + 1;

				self.information
					.entry(face)
					.or_default()
					.set_outgoing(value);
			}
		}
	}

	fn add_bad_node(&self, w: &mut dyn Write, id: Id) -> Result<()> {
		write!(w, "{id} ")?;

		self.information[&id].write(w, id, "BAD NODE")
	}
}

impl<'a, S: Label> Dot<'a, S> {
	fn add_nodes_incoming(&mut self, w: &mut dyn Write, id: Id) -> Result<()> {
		self.graph.predecessors[id]
			.iter()
			.copied()
			.map(Link::node)
			.try_for_each(|n| self.add_node(w, n))
	}

	fn add_gamma(&mut self, w: &mut dyn Write, regions: &[Region], id: Id) -> Result<()> {
		Named::Gamma.write(w, id, |w| {
			self.information[&id].write(w, id, "Selector")?;

			regions.iter().enumerate().try_for_each(|(i, v)| {
				Labeled::new(Named::Then, i).write(w, v.start(), |w| {
					self.add_node(w, v.start())?;
					self.add_node(w, v.end())
				})
			})
		})?;

		self.graph.add_links_incoming(w, id)
	}

	fn add_region(&mut self, w: &mut dyn Write, region: Region, typ: Named, id: Id) -> Result<()> {
		typ.write(w, id, |w| {
			self.add_node(w, region.start())?;
			self.add_node(w, region.end())
		})?;

		self.graph.add_links_redirected(w, region.start(), id)
	}

	fn add_compound(&mut self, w: &mut dyn Write, compound: Compound, id: Id) -> Result<()> {
		let regions = &self.graph.regions[&id];

		match compound {
			Compound::Gamma => self.add_gamma(w, regions, id),
			Compound::Theta => self.add_region(w, regions[0], Named::Theta, id),
			Compound::Lambda => self.add_region(w, regions[0], Named::Lambda, id),
			Compound::Phi => self.add_region(w, regions[0], Named::Phi, id),
		}
	}

	fn add_node(&mut self, w: &mut dyn Write, id: Id) -> Result<()> {
		if self.seen[id] {
			return Ok(());
		}

		self.seen[id] = true;

		let Some(node) = self.graph.nodes.get(id) else { return self.add_bad_node(w, id) };

		self.add_nodes_incoming(w, id)?;

		match *node {
			Node::Simple(ref s) => {
				self.information[&id].write(w, id, Ref(s))?;

				self.graph.add_links_incoming(w, id)
			}
			Node::Marker(m) => {
				self.information[&id].write(w, id, m)?;

				self.graph.add_links_incoming(w, id)
			}
			Node::Compound(c) => self.add_compound(w, c, id),
		}
	}

	fn add_reachable<I>(&mut self, w: &mut dyn Write, roots: I) -> Result<()>
	where
		I: IntoIterator<Item = Id>,
	{
		Named::Reachable.write(w, "reachable", |w| {
			roots.into_iter().try_for_each(|v| self.add_node(w, v))
		})
	}

	fn add_not_unreachable(&mut self, w: &mut dyn Write) -> Result<()> {
		Named::NotReachable.write(w, "unreachable", |w| {
			self.graph
				.nodes
				.keys()
				.try_for_each(|v| self.add_node(w, v))
		})
	}

	/// Writes the graph to the [`Write`] object.
	///
	/// # Errors
	///
	/// If writing to the writer fails.
	pub fn write<I>(&mut self, w: &mut dyn Write, roots: I) -> Result<()>
	where
		I: IntoIterator<Item = Id>,
	{
		const NODE_ATTRIBUTES: &str = r##"shape = plain, style = filled, fillcolor = "#DDDDFF""##;

		writeln!(w, "digraph {{")?;
		writeln!(w, "node [{NODE_ATTRIBUTES}];")?;
		writeln!(w, "style = filled;")?;

		self.initialize_info();

		self.seen.clear();
		self.seen.resize(self.graph.active(), false);

		self.add_reachable(w, roots)?;
		self.add_not_unreachable(w)?;

		writeln!(w, "}}")
	}
}
