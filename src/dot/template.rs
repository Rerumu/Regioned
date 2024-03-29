use std::io::{Result, Write};

use crate::data_flow::{link::Id, node::Compound};

use super::description::Description;

#[derive(Clone, Copy)]
pub enum Group {
	Gamma,
	Theta,
	Lambda,
	Phi,
}

impl Group {
	const fn color(self) -> &'static str {
		match self {
			Self::Gamma => "#a1fc8f",
			Self::Theta => "#eb8582",
			Self::Lambda => "#8bb2f9",
			Self::Phi => "#ecb084",
		}
	}

	const fn label(self) -> &'static str {
		match self {
			Self::Gamma => "Gamma",
			Self::Theta => "Theta",
			Self::Lambda => "Lambda",
			Self::Phi => "Phi",
		}
	}

	pub fn write(self, w: &mut dyn Write) -> Result<()> {
		writeln!(w, r#"fillcolor = "{}";"#, self.color())?;
		writeln!(w, r#"label = "{}";"#, self.label())
	}
}

impl From<&Compound> for Group {
	fn from(value: &Compound) -> Self {
		match value {
			Compound::Gamma { .. } => Self::Gamma,
			Compound::Theta { .. } => Self::Theta,
			Compound::Lambda { .. } => Self::Lambda,
			Compound::Phi { .. } => Self::Phi,
		}
	}
}

#[derive(Clone, Copy)]
pub enum Anchor {
	In,
	Out,
}

impl Anchor {
	pub const fn side(self) -> &'static str {
		match self {
			Self::In => "in",
			Self::Out => "out",
		}
	}

	pub const fn direction(self) -> &'static str {
		match self {
			Self::In => "n",
			Self::Out => "s",
		}
	}

	pub fn write(self, w: &mut dyn Write, id: Id, port: u16) -> Result<()> {
		let side = self.side();
		let direction = self.direction();

		write!(w, "{id}:{side}{port}:{direction}")
	}

	fn write_list<F>(self, w: &mut dyn Write, len: usize, function: F) -> Result<()>
	where
		F: Fn(&mut dyn Write, usize) -> Result<()>,
	{
		write!(w, r#"<TR><TD><TABLE CELLSPACING="0"><TR>"#)?;

		for i in 0..len {
			write!(w, r#"<TD PORT="{}{i}">"#, self.side())?;

			function(w, i)?;

			write!(w, "</TD>")?;
		}

		write!(w, "</TR></TABLE></TD></TR>")
	}
}

#[derive(Default)]
pub struct PortCounts {
	inward: usize,
	outward: usize,
}

impl PortCounts {
	pub fn set_inward(&mut self, inward: usize) {
		self.inward = self.inward.max(inward);
	}

	pub fn set_outward(&mut self, outward: usize) {
		self.outward = self.outward.max(outward);
	}

	fn write_pre_table<T: Description>(&self, w: &mut dyn Write, node: &T) -> Result<()> {
		write!(w, r#"<TABLE BORDER="0" CELLPADDING="0" CELLSPACING="0">"#)?;

		if self.inward > 1 {
			Anchor::In.write_list(w, self.inward, |w, port| node.write_port_in(w, port))?;
		}

		write!(w, "<TR><TD>")
	}

	fn write_post_table<T: Description>(&self, w: &mut dyn Write, node: &T) -> Result<()> {
		write!(w, "</TD></TR>")?;

		if self.outward > 1 {
			Anchor::Out.write_list(w, self.outward, |w, port| node.write_port_out(w, port))?;
		}

		write!(w, "</TABLE>")
	}

	pub fn write<T: Description>(&self, w: &mut dyn Write, node: &T) -> Result<()> {
		let has_table = self.inward > 1 || self.outward > 1;

		write!(w, "[label = <")?;

		if has_table {
			self.write_pre_table(w, node)?;
		}

		write!(w, r#"<TABLE CELLSPACING="0">"#)?;

		node.write_content(w)?;

		write!(w, "</TABLE>")?;

		if has_table {
			self.write_post_table(w, node)?;
		}

		writeln!(w, ">];")
	}
}
