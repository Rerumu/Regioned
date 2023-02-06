use std::{
	fmt::{Display, Formatter, Result as FResult},
	io::{Result as IResult, Write},
	num::NonZeroUsize,
};

use crate::data_flow::node::NodeId;

#[derive(Clone, Copy)]
pub enum Face {
	Incoming,
	Outgoing,
}

impl Face {
	pub fn direction(self) -> &'static str {
		match self {
			Self::Incoming => "n",
			Self::Outgoing => "s",
		}
	}

	pub fn name(self) -> &'static str {
		match self {
			Self::Incoming => "in",
			Self::Outgoing => "out",
		}
	}
}

struct Ports {
	face: Face,
	len: NonZeroUsize,
}

impl Ports {
	fn new(face: Face, len: usize) -> Option<Self> {
		NonZeroUsize::new(len).map(|len| Self { face, len })
	}
}

impl Display for Ports {
	fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
		write!(f, "{{")?;

		for i in 0..self.len.get() {
			if i != 0 {
				write!(f, "| ")?;
			}

			write!(f, "<{}{i}>{i}", self.face.name())?;
		}

		write!(f, "}}")
	}
}

#[derive(Default)]
pub struct Information {
	incoming: usize,
	outgoing: usize,
}

impl Information {
	pub fn set_incoming(&mut self, value: usize) {
		self.incoming = self.incoming.max(value);
	}

	pub fn set_outgoing(&mut self, value: usize) {
		self.outgoing = self.outgoing.max(value);
	}

	pub fn write<T>(&self, w: &mut dyn Write, id: NodeId, label: T) -> IResult<()>
	where
		T: Display,
	{
		write!(w, "{id} [label = \"{{")?;

		if let Some(ports) = Ports::new(Face::Incoming, self.incoming) {
			write!(w, "{ports} | ")?;
		}

		write!(w, "{label}")?;

		if let Some(ports) = Ports::new(Face::Outgoing, self.outgoing) {
			write!(w, " | {ports}")?;
		}

		writeln!(w, "}}\"];")
	}
}
