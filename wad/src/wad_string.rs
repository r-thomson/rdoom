use crate::lump_parser::ParseError;
use std::any::type_name;
use std::fmt;
use std::str::FromStr;

/// The string format used for the name of lumps. It is an 8-byte-long ASCII
/// string, right-padded with null bytes.
///
/// ```
/// # use wad::WadString;
/// let wad_str = WadString::from_bytes(*b"PLAYPAL\0").unwrap();
/// assert_eq!(wad_str.to_string(), "PLAYPAL");
/// ```
#[derive(Debug, PartialEq)]
pub struct WadString {
	bytes: [u8; 8],
}

impl WadString {
	pub fn from_bytes(bytes: [u8; 8]) -> Result<Self, ParseError> {
		if bytes.iter().any(|byte| !byte.is_ascii()) {
			return Err(ParseError::InvalidString);
		}

		Ok(Self { bytes })
	}
}

impl FromStr for WadString {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// This is safe because we will check that each character is ASCII
		let s = s.as_bytes();

		if s.len() > 8 {
			return Err(ParseError::InvalidString);
		}
		if s.iter().any(|byte| !byte.is_ascii()) {
			return Err(ParseError::InvalidString);
		}

		let mut bytes = [0u8; 8];
		bytes[..s.len()].clone_from_slice(s);

		Ok(Self { bytes })
	}
}

impl fmt::Display for WadString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.bytes
			.iter()
			.map_while(|byte| match byte {
				0 => None, // end of string
				1..=127 => Some(*byte as char),
				_ => panic!("Invalid (non-ASCII) character in {}", type_name::<Self>()),
			})
			.collect::<String>()
			.fmt(f)
	}
}

impl PartialEq<&str> for WadString {
	fn eq(&self, other: &&str) -> bool {
		self.to_string() == *other
	}
}

/// Shortcut for initializing a `WadString` from a string literal.
#[macro_export]
macro_rules! wad_string {
	($lit:literal) => {{ WadString::from_str($lit).unwrap() }};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn wad_string_from_bytes_returns_ok() {
		WadString::from_bytes(*b"MYSTRING").unwrap();
	}

	#[test]
	fn wad_string_from_bytes_returns_err_on_invalid_ascii() {
		WadString::from_bytes(*b"INVALID\x80").unwrap_err();
	}

	#[test]
	fn wad_string_from_str_returns_ok() {
		WadString::from_str("MYSTRING").unwrap();
	}

	#[test]
	fn wad_string_from_str_returns_err_on_long_str() {
		WadString::from_str("TOODAMNLONG").unwrap_err();
	}

	#[test]
	fn wad_string_from_str_returns_err_on_invalid_ascii() {
		WadString::from_str("INVALID§").unwrap_err();
	}

	#[test]
	fn wad_string_display() {
		let wad_str = WadString::from_bytes(*b"COLORMAP").unwrap();
		assert_eq!(format!("{}", wad_str), "COLORMAP");

		let wad_str = WadString::from_bytes(*b"DEMO1\0\0\0").unwrap();
		assert_eq!(format!("{}", wad_str), "DEMO1");
	}
}
