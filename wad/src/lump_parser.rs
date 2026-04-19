use crate::{WadString, WadStringError};
use std::result;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
	#[error("unexpected end of lump")]
	EndOfLump,
	#[error("lump contains more bytes than expected")]
	ExtraBytes,
	#[error("invalid WAD string")]
	InvalidString(WadStringError),
}

pub type Result<T> = result::Result<T, ParseError>;

pub(crate) struct LumpParser<'a> {
	remaining: &'a [u8],
}

impl<'a> LumpParser<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self { remaining: data }
	}

	/// Takes the next N bytes without returning anything.
	pub fn read_padding(&mut self, n: usize) -> Result<()> {
		let Some(rest) = self.remaining.get(n..) else {
			self.remaining = &[];
			return Err(ParseError::EndOfLump);
		};
		self.remaining = rest;
		Ok(())
	}

	/// Takes the next N bytes as a non-copied slice.
	pub fn read_slice(&mut self, n: usize) -> Result<&[u8]> {
		let Some((chunk, rest)) = self.remaining.split_at_checked(n) else {
			self.remaining = &[];
			return Err(ParseError::EndOfLump);
		};
		self.remaining = rest;
		Ok(chunk)
	}

	/// Takes the next N bytes as a fixed-size array.
	pub fn read_chunk<const N: usize>(&mut self) -> Result<[u8; N]> {
		let slice = self.read_slice(N)?;
		Ok(slice.try_into().unwrap())
	}

	/// Takes the next byte as an unsigned integer.
	pub fn read_u8(&mut self) -> Result<u8> {
		let bytes = self.read_chunk::<1>()?;
		Ok(bytes[0])
	}

	/// Takes the next 2 bytes as a little-endian signed integer.
	pub fn read_i16(&mut self) -> Result<i16> {
		let bytes = self.read_chunk::<2>()?;
		Ok(i16::from_le_bytes(bytes))
	}

	/// Takes the next 2 bytes as a little-endian unsigned integer.
	pub fn read_u16(&mut self) -> Result<u16> {
		let bytes = self.read_chunk::<2>()?;
		Ok(u16::from_le_bytes(bytes))
	}

	/// Takes the next 4 bytes as a little-endian signed integer.
	pub fn read_i32(&mut self) -> Result<i32> {
		let bytes = self.read_chunk::<4>()?;
		Ok(i32::from_le_bytes(bytes))
	}

	/// Takes the next 4 bytes as a little-endian unsigned integer.
	pub fn read_u32(&mut self) -> Result<u32> {
		let bytes = self.read_chunk::<4>()?;
		Ok(u32::from_le_bytes(bytes))
	}

	/// Takes the next 8 bytes as a `WadString`.
	pub fn read_string(&mut self) -> Result<WadString> {
		return WadString::from_bytes(self.read_chunk()?).map_err(ParseError::InvalidString);
	}

	/// Checks if there are unread bytes remaining.
	pub fn has_data_left(&self) -> bool {
		!self.remaining.is_empty()
	}

	/// Consumes the parser and returns an error if there are unread bytes.
	pub fn finish(self) -> Result<()> {
		if self.has_data_left() {
			Err(ParseError::ExtraBytes)
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::wad_string;

	use super::*;

	#[test]
	fn read_padding_ok() {
		let mut parser = LumpParser::new(b"01234567");

		parser.read_padding(1).unwrap();
		parser.read_padding(2).unwrap();
		parser.read_padding(3).unwrap();
		assert_eq!(parser.read_slice(2).unwrap(), b"67");
	}

	#[test]
	fn read_padding_err() {
		let mut parser = LumpParser::new(b"01234567");

		assert_eq!(parser.read_slice(7).unwrap(), b"0123456");
		assert_eq!(parser.read_padding(2).unwrap_err(), ParseError::EndOfLump);
		assert_eq!(parser.read_padding(1).unwrap_err(), ParseError::EndOfLump);
	}

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
		assert_eq!(parser.read_slice(2).unwrap_err(), ParseError::EndOfLump);
		assert_eq!(parser.read_slice(1).unwrap_err(), ParseError::EndOfLump);
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
		assert_eq!(parser.read_chunk::<2>().unwrap_err(), ParseError::EndOfLump);
		assert_eq!(parser.read_chunk::<1>().unwrap_err(), ParseError::EndOfLump);
	}

	#[test]
	fn read_u8() {
		let data = [0u8, 42u8, u8::MAX];
		let mut parser = LumpParser::new(&data);

		assert_eq!(parser.read_u8().unwrap(), 0);
		assert_eq!(parser.read_u8().unwrap(), 42);
		assert_eq!(parser.read_u8().unwrap(), u8::MAX);
		assert_eq!(parser.read_u8().unwrap_err(), ParseError::EndOfLump);
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
		assert_eq!(parser.read_i16().unwrap_err(), ParseError::EndOfLump);
	}

	#[test]
	fn read_u16() {
		let data = [
			0u16.to_le_bytes(),
			42u16.to_le_bytes(),
			u16::MAX.to_le_bytes(),
		]
		.concat();
		let mut parser = LumpParser::new(&data);

		assert_eq!(parser.read_u16().unwrap(), 0);
		assert_eq!(parser.read_u16().unwrap(), 42);
		assert_eq!(parser.read_u16().unwrap(), u16::MAX);
		assert_eq!(parser.read_u16().unwrap_err(), ParseError::EndOfLump);
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
		assert_eq!(parser.read_i32().unwrap_err(), ParseError::EndOfLump);
	}

	#[test]
	fn read_u32() {
		let data = [
			0u32.to_le_bytes(),
			42u32.to_le_bytes(),
			u32::MAX.to_le_bytes(),
		]
		.concat();
		let mut parser = LumpParser::new(&data);

		assert_eq!(parser.read_u32().unwrap(), 0);
		assert_eq!(parser.read_u32().unwrap(), 42);
		assert_eq!(parser.read_u32().unwrap(), u32::MAX);
		assert_eq!(parser.read_u32().unwrap_err(), ParseError::EndOfLump);
	}

	#[test]
	fn read_string() {
		let data = [*b"COLORMAP", *b"PLAYPAL\0", *b"INVALID\x80"].concat();
		let mut parser = LumpParser::new(&data);

		assert_eq!(parser.read_string().unwrap(), wad_string!("COLORMAP"));
		assert_eq!(parser.read_string().unwrap(), wad_string!("PLAYPAL"));
		assert_eq!(
			parser.read_string().unwrap_err(),
			ParseError::InvalidString(WadStringError::NonAsciiChars)
		);
		assert_eq!(parser.read_string().unwrap_err(), ParseError::EndOfLump);
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
		assert_eq!(parser.finish().unwrap_err(), ParseError::ExtraBytes);
	}
}
