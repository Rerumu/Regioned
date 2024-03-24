#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(u32);

impl Id {
	#[expect(clippy::cast_possible_truncation)]
	#[inline]
	#[must_use]
	pub const fn from_usize(id: usize) -> Self {
		assert!(id as u32 as usize == id, "`usize` should fit in `u32`");

		Self(id as u32)
	}

	#[expect(clippy::cast_possible_truncation)]
	#[inline]
	#[must_use]
	pub const fn into_usize(self) -> usize {
		debug_assert!(
			self.0 as usize as u32 == self.0,
			"`u32` should fit in `usize`"
		);

		self.0 as usize
	}

	pub const MAX: Self = Self(u32::MAX);
	pub const MIN: Self = Self(u32::MIN);
}

impl<T> std::ops::Index<Id> for [T] {
	type Output = T;

	#[inline]
	fn index(&self, id: Id) -> &Self::Output {
		self.index(id.into_usize())
	}
}

impl<T> std::ops::IndexMut<Id> for [T] {
	#[inline]
	fn index_mut(&mut self, id: Id) -> &mut Self::Output {
		self.index_mut(id.into_usize())
	}
}

impl<T> std::ops::Index<Id> for Vec<T> {
	type Output = T;

	#[inline]
	fn index(&self, id: Id) -> &Self::Output {
		self.as_slice().index(id)
	}
}

impl<T> std::ops::IndexMut<Id> for Vec<T> {
	#[inline]
	fn index_mut(&mut self, id: Id) -> &mut Self::Output {
		self.as_mut_slice().index_mut(id)
	}
}

impl Default for Id {
	#[inline]
	fn default() -> Self {
		Self::MAX
	}
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link(pub Id, pub u16);

impl Link {
	#[inline]
	#[must_use]
	pub const fn from_id(id: Id) -> Self {
		Self(id, u16::MIN)
	}

	pub const DANGLING: Self = Self(Id::MAX, u16::MAX);
}

impl From<Id> for Link {
	#[inline]
	fn from(id: Id) -> Self {
		Self::from_id(id)
	}
}
