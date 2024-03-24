use std::io::{Result, Write};

use super::description::Description;

#[derive(Clone, Copy)]
pub struct Ports {
	input: u16,
	output: u16,
}

impl Ports {
	pub const fn new(input: u16, output: u16) -> Self {
		Self { input, output }
	}

	const fn has_both_multiple(self) -> bool {
		self.input > 1 || self.output > 1
	}

	pub fn set_input(&mut self, inputs: usize) {
		self.input = inputs.try_into().unwrap();
	}

	pub fn set_output(&mut self, outputs: u16) {
		self.output = self.output.max(outputs + 1);
	}
}

fn write_port_list<F>(write: &mut dyn Write, len: usize, side: &str, function: F) -> Result<()>
where
	F: Fn(&mut dyn Write, usize) -> Result<()>,
{
	write!(write, "<TR><TD><TABLE CELLSPACING=\"0\"><TR>")?;

	for index in 0..len {
		write!(write, "<TD PORT=\"{side}{index}\">")?;

		function(write, index)?;

		write!(write, "</TD>")?;
	}

	write!(write, "</TR></TABLE></TD></TR>")
}

fn write_input_ports<T>(write: &mut dyn Write, node: &T, input: u16) -> Result<()>
where
	T: Description,
{
	write!(
		write,
		r#"<TABLE BORDER="0" CELLPADDING="0" CELLSPACING="0">"#
	)?;

	if input > 1 {
		write_port_list(write, input.into(), "I", |write, port| {
			node.write_port_in(write, port)
		})?;
	}

	write!(write, "<TR><TD>")
}

fn write_output_ports<T>(write: &mut dyn Write, node: &T, output: u16) -> Result<()>
where
	T: Description,
{
	write!(write, "</TD></TR>")?;

	if output > 1 {
		write_port_list(write, output.into(), "O", |write, port| {
			node.write_port_out(write, port)
		})?;
	}

	write!(write, "</TABLE>")
}

pub fn write_contents<T>(write: &mut dyn Write, node: &T, ports: Ports) -> Result<()>
where
	T: Description,
{
	write!(write, "[label = <")?;

	if ports.has_both_multiple() {
		write_input_ports(write, node, ports.input)?;
	}

	write!(write, "<TABLE CELLSPACING=\"0\">")?;

	node.write_content(write)?;

	write!(write, "</TABLE>")?;

	if ports.has_both_multiple() {
		write_output_ports(write, node, ports.output)?;
	}

	writeln!(write, ">];")
}
