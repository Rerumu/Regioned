use std::{iter::Map, ops::RangeInclusive};

#[cfg(debug_assertions)]
pub type Id = arena::referent::Id<u32, std::num::NonZeroU32>;

#[cfg(not(debug_assertions))]
pub type Id = arena::referent::Id<u32, arena::referent::Nil>;

/// A region with a start and an end marker that delimit it.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region {
	pub start: Id,
	pub end: Id,
}

/// A relationship between an [`Id`] and a [`u16`] port. Two of these together
/// represent a connection between two nodes.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
	pub node: Id,
	pub port: u16,
}

impl Link {
	/// Returns an iterator over all [`Link`]s starting from the current one.
	#[inline]
	pub fn iter(self) -> Map<RangeInclusive<u16>, impl FnMut(u16) -> Self> {
		let node = self.node;
		let iter = self.port..=u16::MAX;

		iter.map(move |port| Self { node, port })
	}
}

impl From<Id> for Link {
	#[inline]
	fn from(node: Id) -> Self {
		Self { node, port: 0 }
	}
}
