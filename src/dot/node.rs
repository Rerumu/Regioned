use std::{
	fmt::{Display, Formatter, Result as FResult},
	io::{Result as IResult, Write},
};

use crate::data_flow::node::Id;

#[derive(Clone, Copy)]
pub enum Face {
	Incoming,
	Outgoing,
}

impl Face {
	pub const fn direction(self) -> &'static str {
		match self {
			Self::Incoming => "n",
			Self::Outgoing => "s",
		}
	}

	pub const fn name(self) -> &'static str {
		match self {
			Self::Incoming => "in",
			Self::Outgoing => "out",
		}
	}
}

struct Ports {
	face: Face,
	len: usize,
}

impl Ports {
	fn new(face: Face, len: usize) -> Option<Self> {
		(len > 1).then_some(Self { face, len })
	}
}

impl Display for Ports {
	fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
		write!(f, "<TR>")?;

		for i in 0..self.len {
			write!(f, r#"<TD PORT="{}{i}">{}</TD>"#, self.face.name(), i + 1)?;
		}

		write!(f, "</TR>")
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

	pub fn write<T>(&self, w: &mut dyn Write, id: Id, label: T) -> IResult<()>
	where
		T: Display,
	{
		write!(w, r#"{id} [label = <<TABLE CELLSPACING="0">"#)?;

		if let Some(ports) = Ports::new(Face::Incoming, self.incoming) {
			write!(w, "{ports}")?;
		}

		let span = self.incoming.max(self.outgoing).max(1);

		write!(w, r#"<TR><TD COLSPAN="{span}">{label}</TD></TR>"#)?;

		if let Some(ports) = Ports::new(Face::Outgoing, self.outgoing) {
			write!(w, "{ports}")?;
		}

		writeln!(w, "</TABLE>>];")
	}
}
