use std::{
	fmt::{Display, Formatter, Result as FResult},
	io::{Result as IResult, Write},
	num::NonZeroUsize,
};

struct Ports {
	prefix: &'static str,
	len: NonZeroUsize,
}

impl Ports {
	fn new(prefix: &'static str, len: usize) -> Option<Self> {
		NonZeroUsize::new(len).map(|len| Self { prefix, len })
	}
}

impl Display for Ports {
	fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
		write!(f, "{{")?;

		for i in 0..self.len.get() {
			if i != 0 {
				write!(f, "| ")?;
			}

			write!(f, "<{}{i}>{i}", self.prefix)?;
		}

		write!(f, "}}")
	}
}

#[derive(Default)]
pub struct Info {
	incoming: usize,
	outgoing: usize,
}

impl Info {
	pub fn set_incoming(&mut self, value: usize) {
		self.incoming = self.incoming.max(value + 1);
	}

	pub fn set_outgoing(&mut self, value: usize) {
		self.outgoing = self.outgoing.max(value + 1);
	}

	pub fn write<T>(&self, w: &mut dyn Write, label: T) -> IResult<()>
	where
		T: Display,
	{
		write!(w, "[label = \"{{")?;

		if let Some(ports) = Ports::new("in", self.incoming) {
			write!(w, "{ports} | ")?;
		}

		write!(w, "{label}")?;

		if let Some(ports) = Ports::new("out", self.outgoing) {
			write!(w, " | {ports}")?;
		}

		writeln!(w, "}}\"];")
	}
}
