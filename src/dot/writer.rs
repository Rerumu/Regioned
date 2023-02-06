use std::{
	collections::{HashMap, HashSet},
	io::{Result, Write},
};

use crate::data_flow::{
	graph::Graph,
	link::Link,
	node::{Compound, Node, NodeId, Region},
};

use super::{
	node::Information,
	region::{Labeled, Named},
	tooltip::{Ref, Tooltip},
};

trait GraphExt<S> {
	fn get_face_region(&self, id: NodeId) -> Option<Region>;

	fn get_face_incoming(&self, id: NodeId) -> NodeId {
		self.get_face_region(id).map_or(id, Region::start)
	}

	fn get_face_outgoing(&self, id: NodeId) -> NodeId {
		self.get_face_region(id).map_or(id, Region::end)
	}

	fn add_link(&self, w: &mut dyn Write, from: Link, to: Link) -> Result<()> {
		let node_a = self.get_face_outgoing(from.node());
		let node_b = self.get_face_incoming(to.node());
		let port_a = from.port().index();
		let port_b = to.port().index();

		writeln!(w, "{node_a}:o{port_a}:s -> {node_b}:i{port_b}:n;")
	}

	fn add_links_redirected(&self, w: &mut dyn Write, to: NodeId, from: NodeId) -> Result<()>;

	fn add_links_incoming(&self, w: &mut dyn Write, to: NodeId) -> Result<()> {
		self.add_links_redirected(w, to, to)
	}
}

impl<S> GraphExt<S> for Graph<S> {
	fn get_face_region(&self, id: NodeId) -> Option<Region> {
		self.nodes[id].as_compound().and_then(|v| match v {
			Compound::Gamma => None,
			_ => self.regions.get(id).and_then(|v| v.first()).copied(),
		})
	}

	fn add_links_redirected(&self, w: &mut dyn Write, to: NodeId, from: NodeId) -> Result<()> {
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

pub struct Writer<'a, S> {
	information: HashMap<NodeId, Information>,
	visited: HashSet<NodeId>,
	graph: &'a Graph<S>,
}

impl<'a, S> Writer<'a, S> {
	/// Creates a new [`Writer`].
	#[must_use]
	pub fn new(graph: &'a Graph<S>) -> Self {
		Self {
			information: HashMap::new(),
			visited: HashSet::new(),
			graph,
		}
	}

	fn initialize_info(&mut self) {
		self.information.clear();

		for (id, list) in &self.graph.predecessors {
			let face = self.graph.get_face_incoming(id);
			let last = list.len();

			self.information.entry(face).or_default().set_incoming(last);

			for link in list {
				let face = self.graph.get_face_outgoing(link.node());
				let last = link.port().into();

				self.information.entry(face).or_default().set_outgoing(last);
			}
		}
	}

	fn add_bad_node(&self, w: &mut dyn Write, id: NodeId) -> Result<()> {
		write!(w, "{id} ")?;

		self.information[&id].write(w, "BAD NODE")
	}
}

impl<'a, S> Writer<'a, S>
where
	S: Tooltip,
{
	fn add_nodes_incoming(&mut self, w: &mut dyn Write, id: NodeId) -> Result<()> {
		self.graph.predecessors[id]
			.iter()
			.copied()
			.map(Link::node)
			.try_for_each(|n| self.add_node(w, n))
	}

	fn add_gamma(&mut self, w: &mut dyn Write, regions: &[Region], id: NodeId) -> Result<()> {
		Named::Gamma.write(w, id, |w| {
			self.information[&id].write(w, "Selector")?;

			regions.iter().enumerate().try_for_each(|(i, v)| {
				Labeled::new(Named::Then, i).write(w, v.start(), |w| {
					self.add_node(w, v.start())?;
					self.add_node(w, v.end())
				})
			})
		})?;

		self.graph.add_links_incoming(w, id)
	}

	fn add_region(
		&mut self,
		w: &mut dyn Write,
		region: Region,
		typ: Named,
		id: NodeId,
	) -> Result<()> {
		typ.write(w, id, |w| {
			self.add_node(w, region.start())?;
			self.add_node(w, region.end())
		})?;

		self.graph.add_links_redirected(w, region.start(), id)
	}

	fn add_compound(&mut self, w: &mut dyn Write, compound: Compound, id: NodeId) -> Result<()> {
		let regions = &self.graph.regions[id];

		match compound {
			Compound::Gamma => self.add_gamma(w, regions, id),
			Compound::Theta => self.add_region(w, regions[0], Named::Theta, id),
			Compound::Lambda => self.add_region(w, regions[0], Named::Lambda, id),
			Compound::Phi => self.add_region(w, regions[0], Named::Phi, id),
		}
	}

	fn add_node(&mut self, w: &mut dyn Write, id: NodeId) -> Result<()> {
		if !self.visited.insert(id) {
			return Ok(());
		}

		let Some(node) = self.graph.nodes.get(id) else { return self.add_bad_node(w, id) };

		self.add_nodes_incoming(w, id)?;

		match *node {
			Node::Simple(ref s) => {
				self.information[&id].write(w, Ref(s))?;

				self.graph.add_links_incoming(w, id)
			}
			Node::Marker(m) => {
				self.information[&id].write(w, m)?;

				self.graph.add_links_incoming(w, id)
			}
			Node::Compound(c) => self.add_compound(w, c, id),
		}
	}

	fn add_reachable(&mut self, w: &mut dyn Write, roots: &[NodeId]) -> Result<()> {
		Named::Reachable.write(w, "reachable", |w| {
			roots.iter().copied().try_for_each(|v| self.add_node(w, v))
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
	pub fn write(&mut self, w: &mut dyn Write, roots: &[NodeId]) -> Result<()> {
		const NODE_ATTRIBUTES: &str = r##"shape = record, style = filled, fillcolor = "#DDDDFF", width = 0, height = 0, margin = "0.05,0.02""##;

		writeln!(w, "digraph {{")?;
		writeln!(w, "node [{NODE_ATTRIBUTES}];")?;
		writeln!(w, "style = filled;")?;

		self.initialize_info();
		self.visited.clear();

		self.add_reachable(w, roots)?;
		self.add_not_unreachable(w)?;

		writeln!(w, "}}")
	}
}
