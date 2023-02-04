use std::{
	fmt::{Display, Formatter, Result as FResult},
	io::{Result as IResult, Write},
};

#[derive(Clone, Copy)]
struct Color(pub Named);

impl Display for Color {
	fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
		let color = match self.0 {
			Named::Gamma => "#8b81e8",
			Named::Theta => "#bb84ca",
			Named::Lambda => "#dde881",
			Named::Phi => "#e6b79a",
			Named::Then => "#89b7d7",
			Named::Reachable => "#81e8bf",
			Named::NotReachable => "#e881aa",
		};

		write!(f, r#""{color}""#)
	}
}

#[derive(Clone, Copy)]
struct Label(pub Named);

impl Display for Label {
	fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
		let name = match self.0 {
			Named::Gamma => "Gamma",
			Named::Theta => "Theta",
			Named::Lambda => "Lambda",
			Named::Phi => "Phi",
			Named::Then => "Then",
			Named::Reachable => "Reachable",
			Named::NotReachable => "Not Reachable",
		};

		write!(f, r#""{name}""#)
	}
}

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
	pub fn write<I, M>(self, w: &mut dyn Write, id: I, mut nested: M) -> IResult<()>
	where
		I: Display,
		M: FnMut(&mut dyn Write) -> IResult<()>,
	{
		writeln!(w, "subgraph cluster_{id} {{")?;
		writeln!(w, "fillcolor = {};", Color(self))?;
		writeln!(w, "label = {};", Label(self))?;

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

	pub fn write<I, M>(&self, w: &mut dyn Write, id: I, mut nested: M) -> IResult<()>
	where
		L: Display,
		I: Display,
		M: FnMut(&mut dyn Write) -> IResult<()>,
	{
		self.typ.write(w, id, |w| {
			writeln!(w, r#"label = "{}";"#, self.label)?;

			nested(w)
		})
	}
}
