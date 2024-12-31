#![allow(missing_docs)]

use alloc::boxed::Box;
use alloc::string::String;
use core::error::Error;
use core::fmt::{self, Display};
use core::ops::Range;
use core::ptr;

#[derive(Debug)]
pub struct Context {
	// The location of the error
	offset: *const u8,
	length: usize,
	// Error itself
	message: Box<dyn Error + Send + Sync + 'static>,
}

impl Display for Context {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.message.fmt(f)
	}
}

impl<E> From<E> for Context
where
	E: Error + Send + Sync + 'static,
{
	fn from(value: E) -> Self {
		Context {
			offset: ptr::null(),
			length: 0,

			message: value.into(),
		}
	}
}

impl Context {
	pub fn new<M>(offset: *const u8, length: usize, message: M) -> Context
	where
		M: Error + Send + Sync + 'static,
	{
		Context {
			offset,
			length,
			message: message.into(),
		}
	}

	pub fn from_message<S: AsRef<str>>(msg: S) -> Context {
		let s = String::from(msg.as_ref());

		Context {
			offset: ptr::null(),
			length: 0,
			message: s.into(),
		}
	}

	pub fn with_span<S>(mut self, slice: S) -> Context
	where
		S: AsRef<[u8]>,
	{
		let slice = slice.as_ref();
		self.offset = slice.as_ptr();
		self.length = slice.len();

		self
	}

	pub fn has_span(&self) -> bool {
		!self.offset.is_null()
	}

	pub fn span(&self) -> Option<Range<usize>> {
		if !self.has_span() {
			return None;
		}
		let offset: usize = self.offset as usize;

		Some(offset..(offset + self.length))
	}

	pub fn downcast_ref<T>(&self) -> Option<&T>
	where
		T: Error + 'static,
	{
		self.message.downcast_ref()
	}
}
