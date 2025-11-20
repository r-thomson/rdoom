use std::result;

// TODO: implement an error type
pub type Result<T> = result::Result<T, ()>;

pub(crate) struct LumpParser<'a> {
	remaining: &'a [u8],
}

impl<'a> LumpParser<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self { remaining: data }
	}

	/// Takes the next N bytes as a non-copied slice.
	pub fn read_slice(&mut self, n: usize) -> Result<&[u8]> {
		let Some((chunk, rest)) = self.remaining.split_at_checked(n) else {
			self.remaining = &[];
			return Err(());
		};
		self.remaining = rest;
		Ok(chunk)
	}

	/// Takes the next N bytes as a fixed-size array.
	pub fn read_chunk<const N: usize>(&mut self) -> Result<[u8; N]> {
		let slice = self.read_slice(N)?;
		Ok(slice.try_into().unwrap())
	}

	/// Takes the next 2 bytes as a little-endian signed integer.
	pub fn read_i16(&mut self) -> Result<i16> {
		let bytes = self.read_chunk::<2>()?;
		Ok(i16::from_le_bytes(bytes))
	}

	/// Takes the next 4 bytes as a little-endian signed integer.
	pub fn read_i32(&mut self) -> Result<i32> {
		let bytes = self.read_chunk::<4>()?;
		Ok(i32::from_le_bytes(bytes))
	}

	/// Checks if there are unread bytes remaining.
	pub fn has_data_left(&self) -> bool {
		!self.remaining.is_empty()
	}

	/// Consumes the parser and returns an error if there are unread bytes.
	pub fn finish(self) -> Result<()> {
		if self.has_data_left() {
			Err(())
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*; // TODO

	#[test]
	fn read_slice_ok() {
		let mut parser = LumpParser::new(b"01234567");

		assert_eq!(parser.read_slice(1).unwrap(), b"0");
		assert_eq!(parser.read_slice(2).unwrap(), b"12");
		assert_eq!(parser.read_slice(3).unwrap(), b"345");
	}

	#[test]
	fn read_slice_err() {
		let mut parser = LumpParser::new(b"01234567");

		assert_eq!(parser.read_slice(7).unwrap(), b"0123456");
		assert!(parser.read_slice(2).is_err());
		assert!(parser.read_slice(1).is_err());
	}

	#[test]
	fn read_chunk_ok() {
		let mut parser = LumpParser::new(b"01234567");

		assert_eq!(parser.read_chunk::<1>().unwrap(), *b"0");
		assert_eq!(parser.read_chunk::<2>().unwrap(), *b"12");
		assert_eq!(parser.read_chunk::<3>().unwrap(), *b"345");
	}

	#[test]
	fn read_chunk_err() {
		let mut parser = LumpParser::new(b"01234567");

		assert_eq!(parser.read_chunk::<7>().unwrap(), *b"0123456");
		assert!(parser.read_chunk::<2>().is_err());
		assert!(parser.read_chunk::<1>().is_err());
	}

	#[test]
	fn read_i16() {
		let data = [
			0i16.to_le_bytes(),
			42i16.to_le_bytes(),
			i16::MAX.to_le_bytes(),
			i16::MIN.to_le_bytes(),
		]
		.concat();
		let mut parser = LumpParser::new(&data);

		assert_eq!(parser.read_i16().unwrap(), 0);
		assert_eq!(parser.read_i16().unwrap(), 42);
		assert_eq!(parser.read_i16().unwrap(), i16::MAX);
		assert_eq!(parser.read_i16().unwrap(), i16::MIN);
	}

	#[test]
	fn read_i32() {
		let data = [
			0i32.to_le_bytes(),
			42i32.to_le_bytes(),
			i32::MAX.to_le_bytes(),
			i32::MIN.to_le_bytes(),
		]
		.concat();
		let mut parser = LumpParser::new(&data);

		assert_eq!(parser.read_i32().unwrap(), 0);
		assert_eq!(parser.read_i32().unwrap(), 42);
		assert_eq!(parser.read_i32().unwrap(), i32::MAX);
		assert_eq!(parser.read_i32().unwrap(), i32::MIN);
	}

	#[test]
	fn has_data_left() {
		let mut parser = LumpParser::new(b"01234567");

		assert!(parser.has_data_left());
		let _ = parser.read_slice(8);
		assert!(!parser.has_data_left());
	}

	#[test]
	fn finish_ok() {
		let mut parser = LumpParser::new(b"01234567");

		let _ = parser.read_slice(8);
		parser.finish().unwrap();
	}

	#[test]
	fn finish_err() {
		let mut parser = LumpParser::new(b"01234567");

		let _ = parser.read_slice(7);
		assert!(parser.finish().is_err());
	}
}
