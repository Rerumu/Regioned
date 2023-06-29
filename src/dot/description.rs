use std::io::{Result, Write};

use crate::data_flow::node::{Compound, Marker, Node};

/// A trait for describing how a node should be written to a DOT file.
pub trait Description {
	/// Write the content of the node onto the label.
	///
	/// # Errors
	///
	/// Returns an error if writing to the writer fails.
	fn write_content(&self, writer: &mut dyn Write) -> Result<()>;

	/// Write the port name of the link to the node.
	///
	/// # Errors
	///
	/// Returns an error if writing to the writer fails.
	fn write_port_in(&self, writer: &mut dyn Write, port: usize) -> Result<()> {
		write!(writer, "{}", port + 1)
	}

	/// Write the port name of the link from the node.
	///
	/// # Errors
	///
	/// Returns an error if writing to the writer fails.
	fn write_port_out(&self, writer: &mut dyn Write, port: usize) -> Result<()> {
		write!(writer, "{}", port + 1)
	}
}

impl Description for Marker {
	fn write_content(&self, writer: &mut dyn Write) -> Result<()> {
		match self {
			Self::Start => write!(writer, "Start"),
			Self::End { .. } => write!(writer, "End"),
		}
	}
}

impl Description for Compound {
	fn write_content(&self, writer: &mut dyn Write) -> Result<()> {
		match self {
			Self::Gamma { .. } => write!(writer, "Gamma"),
			Self::Theta { .. } => write!(writer, "Theta"),
			Self::Lambda { .. } => write!(writer, "Lambda"),
			Self::Phi { .. } => write!(writer, "Phi"),
		}
	}
}

impl<N: Description> Description for Node<N> {
	fn write_content(&self, writer: &mut dyn Write) -> Result<()> {
		match self {
			Self::Simple(simple) => simple.write_content(writer),
			Self::Marker(marker) => marker.write_content(writer),
			Self::Compound(compound) => compound.write_content(writer),
		}
	}

	fn write_port_in(&self, writer: &mut dyn Write, port: usize) -> Result<()> {
		match self {
			Self::Simple(simple) => simple.write_port_in(writer, port),
			Self::Marker(marker) => marker.write_port_in(writer, port),
			Self::Compound(compound) => compound.write_port_in(writer, port),
		}
	}

	fn write_port_out(&self, writer: &mut dyn Write, port: usize) -> Result<()> {
		match self {
			Self::Simple(simple) => simple.write_port_out(writer, port),
			Self::Marker(marker) => marker.write_port_out(writer, port),
			Self::Compound(compound) => compound.write_port_out(writer, port),
		}
	}
}
