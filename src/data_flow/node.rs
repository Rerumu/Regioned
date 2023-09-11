use std::iter::FusedIterator;

use tinyvec::TinyVec;

use super::link::{Link, Region};

macro_rules! impl_iterator {
	($name:tt, $item:ty) => {
		impl<'a, I> Iterator for $name<'a, I>
		where
			I: Iterator<Item = $item>,
		{
			type Item = $item;

			#[inline]
			fn next(&mut self) -> Option<Self::Item> {
				match self {
					Self::List(iter) => iter.next(),
					Self::Opaque(iter) => iter.next(),
				}
			}

			#[inline]
			fn size_hint(&self) -> (usize, Option<usize>) {
				match self {
					Self::List(iter) => iter.size_hint(),
					Self::Opaque(iter) => iter.size_hint(),
				}
			}

			#[inline]
			fn count(self) -> usize {
				match self {
					Self::List(iter) => iter.count(),
					Self::Opaque(iter) => iter.count(),
				}
			}

			#[inline]
			fn last(self) -> Option<Self::Item> {
				match self {
					Self::List(iter) => iter.last(),
					Self::Opaque(iter) => iter.last(),
				}
			}

			#[inline]
			fn nth(&mut self, n: usize) -> Option<Self::Item> {
				match self {
					Self::List(iter) => iter.nth(n),
					Self::Opaque(iter) => iter.nth(n),
				}
			}

			#[inline]
			fn for_each<F>(self, f: F)
			where
				F: FnMut(Self::Item),
			{
				match self {
					Self::List(iter) => iter.for_each(f),
					Self::Opaque(iter) => iter.for_each(f),
				}
			}

			#[inline]
			fn fold<B, F>(self, init: B, f: F) -> B
			where
				F: FnMut(B, Self::Item) -> B,
			{
				match self {
					Self::List(iter) => iter.fold(init, f),
					Self::Opaque(iter) => iter.fold(init, f),
				}
			}
		}

		impl<'a, I> DoubleEndedIterator for $name<'a, I>
		where
			I: DoubleEndedIterator<Item = $item>,
		{
			#[inline]
			fn next_back(&mut self) -> Option<Self::Item> {
				match self {
					Self::List(iter) => iter.next_back(),
					Self::Opaque(iter) => iter.next_back(),
				}
			}

			#[inline]
			fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
				match self {
					Self::List(iter) => iter.nth_back(n),
					Self::Opaque(iter) => iter.nth_back(n),
				}
			}

			#[inline]
			fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
			where
				P: FnMut(&Self::Item) -> bool,
			{
				match self {
					Self::List(iter) => iter.rfind(predicate),
					Self::Opaque(iter) => iter.rfind(predicate),
				}
			}

			#[inline]
			fn rfold<B, F>(self, init: B, f: F) -> B
			where
				F: FnMut(B, Self::Item) -> B,
			{
				match self {
					Self::List(iter) => iter.rfold(init, f),
					Self::Opaque(iter) => iter.rfold(init, f),
				}
			}
		}

		impl<'a, I> ExactSizeIterator for $name<'a, I>
		where
			I: ExactSizeIterator<Item = $item>,
		{
			#[inline]
			fn len(&self) -> usize {
				match self {
					Self::List(iter) => iter.len(),
					Self::Opaque(iter) => iter.len(),
				}
			}
		}

		impl<'a, I> FusedIterator for $name<'a, I> where I: FusedIterator<Item = $item> {}
	};
}

#[derive(Clone)]
pub enum Iter<'a, I> {
	List(std::slice::Iter<'a, Link>),
	Opaque(I),
}

impl_iterator!(Iter, &'a Link);

pub enum IterMut<'a, I> {
	List(std::slice::IterMut<'a, Link>),
	Opaque(I),
}

impl_iterator!(IterMut, &'a mut Link);

/// A marker node.
///
/// It is used to mark the start and end of a region.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Marker {
	/// The "start" node.
	///
	/// It represents the start and arguments to a region.
	Start,

	/// The "end" node.
	///
	/// It represents the end and returns of a region.
	End { parameters: Vec<Link> },
}

impl Marker {
	/// Returns a reference to the parameters of the node.
	#[inline]
	#[must_use]
	pub fn as_parameters(&self) -> Option<&[Link]> {
		match self {
			Self::Start => None,
			Self::End { parameters } => Some(parameters),
		}
	}

	/// Returns a mutable reference to the parameters of the node.
	#[inline]
	#[must_use]
	pub fn as_parameters_mut(&mut self) -> Option<&mut Vec<Link>> {
		match self {
			Self::Start => None,
			Self::End { parameters } => Some(parameters),
		}
	}
}

/// A compound node.
///
/// It is used to represent regions and their parameters.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Compound {
	/// The "select" node.
	///
	/// The node's last input denotes which region to select.
	/// All other values are passed to `start`.
	/// All values passed to `end` are output.
	Gamma {
		parameters: Vec<Link>,
		regions: TinyVec<[Region; 2]>,
	},

	/// The "loop" node.
	///
	/// The node's inputs are passed to `start`.
	/// The `end`'s last input decides whether to loop.
	/// All other values are either passed to `start` or output.
	Theta {
		parameters: Vec<Link>,
		region: Region,
	},

	/// The "function" node.
	///
	/// The node's inputs are passed to `start`.
	/// The function itself is output.
	Lambda {
		parameters: Vec<Link>,
		region: Region,
	},

	/// The "mutual recursion" node.
	///
	/// The node's inputs are passed to `start`.
	/// The `end`'s inputs are passed to `start`.
	Phi {
		parameters: Vec<Link>,
		region: Region,
	},
}

impl Compound {
	/// Returns a reference to the parameters of the node.
	#[inline]
	#[must_use]
	pub fn as_parameters(&self) -> &[Link] {
		match self {
			Self::Gamma { parameters, .. }
			| Self::Theta { parameters, .. }
			| Self::Lambda { parameters, .. }
			| Self::Phi { parameters, .. } => parameters,
		}
	}

	/// Returns a mutable reference to the parameters of the node.
	#[inline]
	#[must_use]
	pub fn as_parameters_mut(&mut self) -> &mut Vec<Link> {
		match self {
			Self::Gamma { parameters, .. }
			| Self::Theta { parameters, .. }
			| Self::Lambda { parameters, .. }
			| Self::Phi { parameters, .. } => parameters,
		}
	}

	/// Returns the regions of the node.
	#[inline]
	#[must_use]
	pub fn regions(&self) -> &[Region] {
		match self {
			Self::Gamma { regions, .. } => regions,
			Self::Theta { region, .. } | Self::Lambda { region, .. } | Self::Phi { region, .. } => {
				std::slice::from_ref(region)
			}
		}
	}
}

/// A node in the data flow graph.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node<N> {
	Simple(N),
	Marker(Marker),
	Compound(Compound),
}

impl<N> Node<N> {
	/// Returns a reference to the [`Node::Simple`] node if it is one.
	#[inline]
	#[must_use]
	pub const fn as_simple(&self) -> Option<&N> {
		if let Self::Simple(simple) = self {
			Some(simple)
		} else {
			None
		}
	}

	/// Returns a mutable reference to the [`Node::Simple`] node if it is one.
	#[inline]
	#[must_use]
	pub fn as_simple_mut(&mut self) -> Option<&mut N> {
		if let Self::Simple(simple) = self {
			Some(simple)
		} else {
			None
		}
	}

	/// Returns a reference to the [`Node::Marker`] node if it is one.
	#[inline]
	#[must_use]
	pub const fn as_marker(&self) -> Option<&Marker> {
		if let Self::Marker(marker) = self {
			Some(marker)
		} else {
			None
		}
	}

	/// Returns a mutable reference to the [`Node::Marker`] node if it is one.
	#[inline]
	#[must_use]
	pub fn as_marker_mut(&mut self) -> Option<&mut Marker> {
		if let Self::Marker(marker) = self {
			Some(marker)
		} else {
			None
		}
	}

	/// Returns a reference to the [`Node::Compound`] node if it is one.
	#[inline]
	#[must_use]
	pub const fn as_compound(&self) -> Option<&Compound> {
		if let Self::Compound(compound) = self {
			Some(compound)
		} else {
			None
		}
	}

	/// Returns a mutable reference to the [`Node::Compound`] node if it is one.
	#[inline]
	#[must_use]
	pub fn as_compound_mut(&mut self) -> Option<&mut Compound> {
		if let Self::Compound(compound) = self {
			Some(compound)
		} else {
			None
		}
	}
}

/// A node that can represent parameters as an array of [`Link`]s.
pub trait AsParameters {
	/// Returns a reference to the parameters of the node.
	#[must_use]
	fn as_parameters(&self) -> Option<&[Link]>;
}

impl<N: AsParameters> Node<N> {
	#[inline]
	#[must_use]
	pub fn as_parameters(&self) -> Option<&[Link]> {
		match self {
			Self::Simple(simple) => simple.as_parameters(),
			Self::Marker(marker) => marker.as_parameters(),
			Self::Compound(compound) => Some(compound.as_parameters()),
		}
	}
}

/// A node that can represent parameters a mutable [`Vec<Link>`].
pub trait AsParametersMut {
	/// Returns a mutable reference to the parameters of the node.
	#[must_use]
	fn as_parameters_mut(&mut self) -> Option<&mut Vec<Link>>;
}

impl<N: AsParametersMut> Node<N> {
	#[inline]
	#[must_use]
	pub fn as_parameters_mut(&mut self) -> Option<&mut Vec<Link>> {
		match self {
			Self::Simple(simple) => simple.as_parameters_mut(),
			Self::Marker(marker) => marker.as_parameters_mut(),
			Self::Compound(compound) => Some(compound.as_parameters_mut()),
		}
	}
}

/// A node that can represent parameters as an iterator of [`Link`] referencess.
pub trait Parameters {
	type Iter<'a>: Iterator<Item = &'a Link>
	where
		Self: 'a;

	/// Returns an iterator over the parameters of the node.
	#[must_use]
	fn parameters(&self) -> Self::Iter<'_>;
}

impl<N: Parameters> Node<N> {
	#[inline]
	#[must_use]
	pub fn parameters(&self) -> Iter<N::Iter<'_>> {
		let dead = &[][..];

		match self {
			Self::Simple(simple) => Iter::Opaque(simple.parameters()),
			Self::Marker(marker) => Iter::List(marker.as_parameters().unwrap_or(dead).iter()),
			Self::Compound(compound) => Iter::List(compound.as_parameters().iter()),
		}
	}
}

/// A node that can represent parameters as an iterator of mutable [`Link`] references.
pub trait ParametersMut {
	type Iter<'a>: Iterator<Item = &'a mut Link>
	where
		Self: 'a;

	/// Returns an iterator over the parameters of the node.
	#[must_use]
	fn parameters_mut(&mut self) -> Self::Iter<'_>;
}

impl<N: ParametersMut> Node<N> {
	#[inline]
	#[must_use]
	pub fn parameters_mut(&mut self) -> IterMut<N::Iter<'_>> {
		let dead = &mut [][..];

		match self {
			Self::Simple(simple) => IterMut::Opaque(simple.parameters_mut()),
			Self::Marker(marker) => {
				let iter = marker.as_parameters_mut().map_or(dead, Vec::as_mut_slice);

				IterMut::List(iter.iter_mut())
			}
			Self::Compound(compound) => IterMut::List(compound.as_parameters_mut().iter_mut()),
		}
	}
}
