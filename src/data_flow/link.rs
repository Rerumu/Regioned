use std::{iter::Map, ops::RangeInclusive};

#[cfg(debug_assertions)]
pub type Id = arena::key::Id<std::num::NonZeroU32>;

#[cfg(not(debug_assertions))]
pub type Id = arena::key::Id<arena::version::Nil>;

/// A region with a start and an end marker that delimit it.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region {
	start: Id,
	end: Id,
}

impl Region {
	pub(crate) const fn new(start: Id, end: Id) -> Self {
		Self { start, end }
	}

	/// Returns the start marker [`Id`] of the region.
	#[inline]
	#[must_use]
	pub const fn start(self) -> Id {
		self.start
	}

	/// Returns the end marker [`Id`] of the region.
	#[inline]
	#[must_use]
	pub const fn end(self) -> Id {
		self.end
	}
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
