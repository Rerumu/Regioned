use std::{
	fmt::Display,
	io::{Result, Write},
};

#[derive(Clone, Copy)]
pub enum Named {
	Gamma,
	Theta,
	Lambda,
	Phi,
	Then,
	Reachable,
	NotReachable,
}

impl Named {
	fn color(self) -> &'static str {
		match self {
			Self::Gamma => "#8b81e8",
			Self::Theta => "#bb84ca",
			Self::Lambda => "#dde881",
			Self::Phi => "#e6b79a",
			Self::Then => "#89b7d7",
			Self::Reachable => "#81e8bf",
			Self::NotReachable => "#e881aa",
		}
	}

	fn label(self) -> &'static str {
		match self {
			Self::Gamma => "Gamma",
			Self::Theta => "Theta",
			Self::Lambda => "Lambda",
			Self::Phi => "Phi",
			Self::Then => "Then",
			Self::Reachable => "Reachable",
			Self::NotReachable => "Not Reachable",
		}
	}

	pub fn write<I, M>(self, w: &mut dyn Write, id: I, nested: M) -> Result<()>
	where
		I: Display,
		M: FnOnce(&mut dyn Write) -> Result<()>,
	{
		writeln!(w, "subgraph cluster_{id} {{")?;
		writeln!(w, r#"fillcolor = "{}";"#, self.color())?;
		writeln!(w, r#"label = "{}";"#, self.label())?;

		nested(w)?;

		writeln!(w, "}}")
	}
}

pub struct Labeled<L> {
	typ: Named,
	label: L,
}

impl<L> Labeled<L> {
	pub const fn new(typ: Named, label: L) -> Self {
		Self { typ, label }
	}

	pub fn write<I, M>(&self, w: &mut dyn Write, id: I, nested: M) -> Result<()>
	where
		L: Display,
		I: Display,
		M: FnOnce(&mut dyn Write) -> Result<()>,
	{
		self.typ.write(w, id, |w| {
			writeln!(w, r#"label = "{}";"#, self.label)?;

			nested(w)
		})
	}
}
