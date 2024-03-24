use tinyvec::TinyVec;

use super::link::Link;

macro_rules! impl_mirrored {
	($item:expr, $iter:pat => $apply:expr) => {
		match $item {
			Self::List($iter) => $apply,
			Self::Simple($iter) => $apply,
		}
	};
}

macro_rules! impl_iterator {
	($name:tt, $item:ty) => {
		impl<'a, I: Iterator<Item = $item>> Iterator for $name<'a, I> {
			type Item = $item;

			#[inline]
			fn next(&mut self) -> Option<Self::Item> {
				impl_mirrored!(self, iter => iter.next())
			}

			#[inline]
			fn size_hint(&self) -> (usize, Option<usize>) {
				impl_mirrored!(self, iter => iter.size_hint())
			}

			#[inline]
			fn count(self) -> usize {
				impl_mirrored!(self, iter => iter.count())
			}

			#[inline]
			fn last(self) -> Option<Self::Item> {
				impl_mirrored!(self, iter => iter.last())
			}

			#[inline]
			fn nth(&mut self, n: usize) -> Option<Self::Item> {
				impl_mirrored!(self, iter => iter.nth(n))
			}

			#[inline]
			fn for_each<F>(self, f: F)
			where
				F: FnMut(Self::Item),
			{
				impl_mirrored!(self, iter => iter.for_each(f))
			}

			#[inline]
			fn fold<B, F>(self, init: B, f: F) -> B
			where
				F: FnMut(B, Self::Item) -> B,
			{
				impl_mirrored!(self, iter => iter.fold(init, f))
			}
		}

		impl<'a, I: DoubleEndedIterator<Item = $item>> DoubleEndedIterator for $name<'a, I> {
			#[inline]
			fn next_back(&mut self) -> Option<Self::Item> {
				impl_mirrored!(self, iter => iter.next_back())
			}

			#[inline]
			fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
				impl_mirrored!(self, iter => iter.nth_back(n))
			}

			#[inline]
			fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
			where
				P: FnMut(&Self::Item) -> bool,
			{
				impl_mirrored!(self, iter => iter.rfind(predicate))
			}

			#[inline]
			fn rfold<B, F>(self, init: B, f: F) -> B
			where
				F: FnMut(B, Self::Item) -> B,
			{
				impl_mirrored!(self, iter => iter.rfold(init, f))
			}
		}

		impl<'a, I> ExactSizeIterator for $name<'a, I>
		where
			I: ExactSizeIterator<Item = $item>,
		{
			#[inline]
			fn len(&self) -> usize {
				impl_mirrored!(self, iter => iter.len())
			}
		}

		impl<'a, I> std::iter::FusedIterator for $name<'a, I> where I: std::iter::FusedIterator<Item = $item> {}
	};
}

macro_rules! impl_compound {
	($type:tt, $error:tt) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct $error;

		impl std::error::Error for $error {}

		impl std::fmt::Display for $error {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				std::fmt::Debug::fmt(self, f)
			}
		}

		impl<T> TryFrom<Node<T>> for $type {
			type Error = $error;

			#[inline]
			fn try_from(node: Node<T>) -> Result<Self, Self::Error> {
				match node {
					Node::$type(node) => Ok(node),
					_ => Err($error),
				}
			}
		}

		impl<T> From<$type> for Node<T> {
			#[inline]
			fn from(node: $type) -> Self {
				Self::$type(node)
			}
		}
	};
}

/// The "select" node.
///
/// The node's last parameter denotes which region to select.
/// All other parameters are passed into the user-defined start node.
pub struct Gamma {
	pub parameters: Vec<Link>,
	pub results: TinyVec<[Vec<Link>; 2]>,
}

impl_compound!(Gamma, NotGammaError);

/// The "repeat" node.
///
/// The node's last result denotes whether to repeat the loop.
/// All other parameters and results are passed into the user-defined start node.
pub struct Theta {
	pub parameters: Vec<Link>,
	pub results: Vec<Link>,
}

impl_compound!(Theta, NotThetaError);

/// The "function" node.
///
/// The node's parameters are the function's bound inputs.
/// The node's single result is the function itself.
pub struct Lambda {
	pub parameters: Vec<Link>,
	pub results: Vec<Link>,
}

impl_compound!(Lambda, NotLambdaError);

/// The "mutually recursive" node.
///
/// All parameters and results are passed into the user-defined start node.
pub struct Phi {
	pub parameters: Vec<Link>,
	pub results: Vec<Link>,
}

impl_compound!(Phi, NotPhiError);

pub enum Node<T> {
	Simple(T),
	Gamma(Gamma),
	Theta(Theta),
	Phi(Phi),
	Lambda(Lambda),
}

impl<T> Node<T> {
	/// Returns the node as a `T` reference if it is one.
	#[inline]
	#[must_use]
	pub const fn as_simple(&self) -> Option<&T> {
		match self {
			Self::Simple(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a `T` mutable reference if it is node.
	#[inline]
	#[must_use]
	pub fn as_mut_simple(&mut self) -> Option<&mut T> {
		match self {
			Self::Simple(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Gamma`] reference if it is one.
	#[inline]
	#[must_use]
	pub const fn as_gamma(&self) -> Option<&Gamma> {
		match self {
			Self::Gamma(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Gamma`] mutable reference if it is one.
	#[inline]
	#[must_use]
	pub fn as_mut_gamma(&mut self) -> Option<&mut Gamma> {
		match self {
			Self::Gamma(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Theta`] reference if it is one.
	#[inline]
	#[must_use]
	pub const fn as_theta(&self) -> Option<&Theta> {
		match self {
			Self::Theta(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Theta`] mutable reference if it is one.
	#[inline]
	#[must_use]
	pub fn as_mut_theta(&mut self) -> Option<&mut Theta> {
		match self {
			Self::Theta(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Phi`] reference if it is one.
	#[inline]
	#[must_use]
	pub const fn as_phi(&self) -> Option<&Phi> {
		match self {
			Self::Phi(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Phi`] mutable reference if it is one.
	#[inline]
	#[must_use]
	pub fn as_mut_phi(&mut self) -> Option<&mut Phi> {
		match self {
			Self::Phi(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Lambda`] reference if it is one.
	#[inline]
	#[must_use]
	pub const fn as_lambda(&self) -> Option<&Lambda> {
		match self {
			Self::Lambda(node) => Some(node),
			_ => None,
		}
	}

	/// Returns the node as a [`Lambda`] mutable reference if it is one.
	#[inline]
	#[must_use]
	pub fn as_mut_lambda(&mut self) -> Option<&mut Lambda> {
		match self {
			Self::Lambda(node) => Some(node),
			_ => None,
		}
	}

	/// Returns a reference to the results arrays of the node if it is compound.
	#[inline]
	#[must_use]
	pub fn as_results(&self) -> Option<&[Vec<Link>]> {
		let result = match self {
			Self::Simple(_) => return None,
			Self::Gamma(node) => &node.results,
			Self::Theta(node) => std::slice::from_ref(&node.results),
			Self::Phi(node) => std::slice::from_ref(&node.results),
			Self::Lambda(node) => std::slice::from_ref(&node.results),
		};

		Some(result)
	}

	/// Returns a mutable reference to the results arrays of the node if it is compound.
	#[inline]
	#[must_use]
	pub fn as_mut_results(&mut self) -> Option<&mut [Vec<Link>]> {
		let result = match self {
			Self::Simple(_) => return None,
			Self::Gamma(node) => &mut node.results,
			Self::Theta(node) => std::slice::from_mut(&mut node.results),
			Self::Phi(node) => std::slice::from_mut(&mut node.results),
			Self::Lambda(node) => std::slice::from_mut(&mut node.results),
		};

		Some(result)
	}

	/// Returns a reference to the parameters of the node if it is compound.
	#[inline]
	#[must_use]
	pub fn as_parameters(&self) -> Option<&[Link]> {
		let result = match self {
			Self::Simple(_) => return None,
			Self::Gamma(node) => &node.parameters,
			Self::Theta(node) => &node.parameters,
			Self::Phi(node) => &node.parameters,
			Self::Lambda(node) => &node.parameters,
		};

		Some(result)
	}

	/// Returns a mutable reference to the parameters of the node if it is compound.
	#[inline]
	#[must_use]
	pub fn as_mut_parameters(&mut self) -> Option<&mut Vec<Link>> {
		let result = match self {
			Self::Simple(_) => return None,
			Self::Gamma(node) => &mut node.parameters,
			Self::Theta(node) => &mut node.parameters,
			Self::Phi(node) => &mut node.parameters,
			Self::Lambda(node) => &mut node.parameters,
		};

		Some(result)
	}
}

/// A node that can represent parameters as an iterator of [`Link`] referencess.
pub trait Parameters {
	type Iter<'a>: Iterator<Item = &'a Link> + 'a
	where
		Self: 'a;

	/// Returns an iterator over the parameters of the node.
	#[must_use]
	fn parameters(&self) -> Self::Iter<'_>;
}

pub enum Iter<'a, T> {
	List(std::slice::Iter<'a, Link>),
	Simple(T),
}

impl_iterator!(Iter, &'a Link);

impl<T: Parameters> Node<T> {
	/// Returns an iterator over the parameters of the node, including simple nodes.
	#[inline]
	#[must_use]
	pub fn parameters(&self) -> Iter<'_, T::Iter<'_>> {
		let iter = match self {
			Self::Simple(node) => return Iter::Simple(node.parameters()),
			Self::Gamma(node) => node.parameters.iter(),
			Self::Theta(node) => node.parameters.iter(),
			Self::Phi(node) => node.parameters.iter(),
			Self::Lambda(node) => node.parameters.iter(),
		};

		Iter::List(iter)
	}
}

/// A node that can represent parameters as an iterator of mutable [`Link`] references.
pub trait ParametersMut {
	type IterMut<'a>: Iterator<Item = &'a mut Link> + 'a
	where
		Self: 'a;

	/// Returns an iterator over the parameters of the node.
	#[must_use]
	fn parameters_mut(&mut self) -> Self::IterMut<'_>;
}

pub enum IterMut<'a, T> {
	List(std::slice::IterMut<'a, Link>),
	Simple(T),
}

impl_iterator!(IterMut, &'a mut Link);

impl<T: ParametersMut> Node<T> {
	/// Returns a mutable iterator over the parameters of the node, including simple nodes.
	#[inline]
	#[must_use]
	pub fn parameters_mut(&mut self) -> IterMut<'_, T::IterMut<'_>> {
		let iter = match self {
			Self::Simple(node) => return IterMut::Simple(node.parameters_mut()),
			Self::Gamma(node) => node.parameters.iter_mut(),
			Self::Theta(node) => node.parameters.iter_mut(),
			Self::Phi(node) => node.parameters.iter_mut(),
			Self::Lambda(node) => node.parameters.iter_mut(),
		};

		IterMut::List(iter)
	}
}
