use std::io::{Result, Write};

use crate::collection::node::Node;

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

impl Description for usize {
	fn write_content(&self, writer: &mut dyn Write) -> Result<()> {
		write!(writer, "<TR><TD>{self}</TD></TR>")
	}
}

impl<T: Description> Description for Node<T> {
	fn write_content(&self, writer: &mut dyn Write) -> Result<()> {
		let name = match self {
			Self::Simple(node) => return node.write_content(writer),
			Self::Gamma(_) => "Gamma",
			Self::Theta(_) => "Theta",
			Self::Phi(_) => "Phi",
			Self::Lambda(_) => "Lambda",
		};

		write!(writer, "<TR><TD>{name}</TD></TR>")
	}

	fn write_port_in(&self, writer: &mut dyn Write, port: usize) -> Result<()> {
		if let Self::Simple(node) = self {
			node.write_port_in(writer, port)
		} else {
			write!(writer, "{}", port + 1)
		}
	}

	fn write_port_out(&self, writer: &mut dyn Write, port: usize) -> Result<()> {
		if let Self::Simple(node) = self {
			node.write_port_out(writer, port)
		} else {
			write!(writer, "{}", port + 1)
		}
	}
}
