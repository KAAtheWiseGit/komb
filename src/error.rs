#![allow(missing_docs)]

use alloc::{vec, vec::Vec};
use core::fmt::{self, Display};

use crate::Context;

#[macro_export]
macro_rules! bail {
	($err:expr) => {
		return Err(Context::from_error($err).into())
	};
}

#[derive(Debug)]
pub struct Error {
	// INVARIANT: must have at least one element
	stack: Vec<Context>,
}

impl From<Context> for Error {
	fn from(value: Context) -> Self {
		Error { stack: vec![value] }
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, context) in self.stack.iter().enumerate().rev() {
			f.write_fmt(format_args!("{i}: "))?;
			context.fmt(f)?;
			f.write_str("\n")?;
		}
		Ok(())
	}
}

impl core::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		Some(self.source())
	}
}

impl<'a> IntoIterator for &'a Error {
	type Item = &'a Context;
	type IntoIter = alloc::slice::Iter<'a, Context>;

	fn into_iter(self) -> Self::IntoIter {
		self.stack.iter()
	}
}

impl Error {
	pub fn new<I>(collection: I) -> Error
	where
		I: IntoIterator<Item = Context>,
	{
		Error {
			stack: collection.into_iter().collect(),
		}
	}

	pub fn end<S>(input: S) -> Error
	where
		S: AsRef<[u8]>,
	{
		Context::from_message("Reached the end of input")
			.with_span(input)
			.into()
	}

	pub fn with_context<F>(mut self, f: F) -> Error
	where
		F: FnOnce() -> Context,
	{
		self.stack.push(f());
		self
	}

	pub fn source(&self) -> &Context {
		&self.stack[0]
	}

	pub fn downcast_ref<T>(&self) -> Option<&T>
	where
		T: core::error::Error + 'static,
	{
		for context in &self.stack {
			if let Some(value) = context.downcast_ref() {
				return Some(value);
			}
		}

		None
	}
}
