use tinyvec::TinyVec;

use crate::data_flow::{link::Id, node::Parameters, nodes::Nodes};

use super::reverse_topological::ReverseTopological;

pub type SuccessorList = TinyVec<[Id; 2]>;

/// A node successor finder.
/// It caches the successors for each node after a traversal.
#[derive(Default)]
pub struct Successors {
	cache: Vec<SuccessorList>,
}

impl Successors {
	/// Creates a new, reusable [`Successors`] instance.
	#[must_use]
	pub const fn new() -> Self {
		Self { cache: Vec::new() }
	}

	/// Returns the cached successors.
	#[must_use]
	pub fn cache(&self) -> &[SuccessorList] {
		&self.cache
	}

	/// Clears the cache.
	pub fn clear(&mut self) {
		self.cache.clear();
	}

	/// Finds and caches all successors coming back from the roots.
	pub fn run<S, I>(&mut self, nodes: &Nodes<S>, roots: I, topological: &mut ReverseTopological)
	where
		S: Parameters,
		I: IntoIterator<Item = Id>,
	{
		let active = nodes.active();

		self.cache.iter_mut().for_each(SuccessorList::clear);

		if self.cache.len() < active {
			self.cache.resize_with(active, SuccessorList::new);
		}

		for id in topological.iter(nodes, roots) {
			for predecessor in nodes[id].parameters() {
				let successors = &mut self.cache[predecessor.node];

				if !successors.contains(&id) {
					successors.push(id);
				}
			}
		}
	}
}
