use std::ops::Range;

#[cfg(debug_assertions)]
pub type Id = arena::referent::Id<u32, std::num::NonZeroU64>;

#[cfg(not(debug_assertions))]
pub type Id = arena::referent::Id<u32, arena::referent::Nil>;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
	pub node: Id,
	pub port: u16,
}

impl Link {
	#[inline]
	#[must_use]
	pub const fn iter(self) -> Iter {
		Iter {
			node: self.node,
			ports: self.port..u16::MAX,
		}
	}
}

impl From<Id> for Link {
	#[inline]
	fn from(node: Id) -> Self {
		Self { node, port: 0 }
	}
}

impl IntoIterator for Link {
	type Item = Self;
	type IntoIter = Iter;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Iter {
	pub node: Id,
	pub ports: Range<u16>,
}

impl Iterator for Iter {
	type Item = Link;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.ports.next().map(|port| Link {
			node: self.node,
			port,
		})
	}

	#[inline]
	fn count(self) -> usize {
		self.ports.count()
	}

	#[inline]
	fn last(self) -> Option<Self::Item> {
		self.ports.last().map(|port| Link {
			node: self.node,
			port,
		})
	}

	#[inline]
	fn max(self) -> Option<Self::Item> {
		self.ports.max().map(|port| Link {
			node: self.node,
			port,
		})
	}

	#[inline]
	fn min(self) -> Option<Self::Item> {
		self.ports.min().map(|port| Link {
			node: self.node,
			port,
		})
	}

	#[inline]
	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		self.ports.nth(n).map(|port| Link {
			node: self.node,
			port,
		})
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.ports.size_hint()
	}
}

impl DoubleEndedIterator for Iter {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.ports.next_back().map(|port| Link {
			node: self.node,
			port,
		})
	}

	#[inline]
	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		self.ports.nth_back(n).map(|port| Link {
			node: self.node,
			port,
		})
	}
}

impl ExactSizeIterator for Iter {
	#[inline]
	fn len(&self) -> usize {
		self.ports.len()
	}
}

impl std::iter::FusedIterator for Iter {}
