use std::any::type_name;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

/// Where's All the Data?
#[derive(Debug)]
pub struct Wad {
	file: File,
	pub header: WadHeader,
	pub directory: Vec<WadDirectoryEntry>,
}

impl Wad {
	pub fn new(mut file: File) -> Result<Self, ()> {
		let mut header_buf = [0; WadHeader::SIZE_BYTES];
		file.seek(SeekFrom::Start(0))
			.and_then(|_| file.read_exact(&mut header_buf))
			.map_err(|_| ())?;

		let header = WadHeader::new(header_buf)?;

		let mut directory_buf = vec![0; header.num_lumps as usize * WadDirectoryEntry::SIZE_BYTES];
		file.seek(SeekFrom::Start(header.directory_offset_bytes as u64))
			.and_then(|_| file.read_exact(&mut directory_buf))
			.map_err(|_| ())?;

		let directory: Vec<WadDirectoryEntry> = directory_buf
			.chunks(WadDirectoryEntry::SIZE_BYTES)
			.map(|chunk| chunk.try_into().unwrap())
			.map(WadDirectoryEntry::new)
			.collect::<Result<_, _>>()?;

		Ok(Wad {
			file,
			header,
			directory,
		})
	}
}

#[derive(Debug)]
pub struct WadHeader {
	pub iwad_or_pwad: WadType,
	pub num_lumps: i32,
	pub directory_offset_bytes: i32,
}

impl WadHeader {
	pub const SIZE_BYTES: usize = 12;

	fn new(data: [u8; 12]) -> Result<Self, ()> {
		Ok(WadHeader {
			iwad_or_pwad: WadType::new(data[0..4].try_into().unwrap())?,
			num_lumps: i32::from_le_bytes(data[4..8].try_into().unwrap()),
			directory_offset_bytes: i32::from_le_bytes(data[8..12].try_into().unwrap()),
		})
	}
}

/// Either IWAD or PWAD
#[derive(Debug, Eq, PartialEq)]
pub enum WadType {
	IWAD,
	PWAD,
}

impl WadType {
	pub const SIZE_BYTES: usize = 4;

	pub fn new(data: [u8; Self::SIZE_BYTES]) -> Result<Self, ()> {
		match &data {
			b"IWAD" => Ok(Self::IWAD),
			b"PWAD" => Ok(Self::PWAD),
			_ => Err(()),
		}
	}
}

#[derive(Debug)]
pub struct WadDirectoryEntry {
	pub offset_bytes: i32,
	pub size_bytes: i32,
	pub lump_name: WadString,
}

impl WadDirectoryEntry {
	pub const SIZE_BYTES: usize = 16;

	pub fn new(data: [u8; Self::SIZE_BYTES]) -> Result<Self, ()> {
		Ok(WadDirectoryEntry {
			offset_bytes: i32::from_le_bytes(data[0..4].try_into().unwrap()),
			size_bytes: i32::from_le_bytes(data[4..8].try_into().unwrap()),
			lump_name: WadString::new(data[8..16].try_into().unwrap()).unwrap(),
		})
	}

	/// Virtual lumps have a size of zero and only appear in the directory
	pub fn is_virtual(&self) -> bool {
		self.size_bytes == 0
	}

	/// Read the contents of a lump into a buffer. The buffer's size must equal `size_bytes`.
	pub fn read_lump(&self, buf: &mut [u8], wadfile: &Wad) -> std::io::Result<()> {
		assert!(buf.len() == self.size_bytes as usize);

		let mut file = &wadfile.file;

		file.seek(SeekFrom::Start(self.offset_bytes as u64))?;
		file.read_exact(buf)?;

		Ok(())
	}
}

/// The string format used for the name of lumps. It is an 8-byte-long ASCII
/// string, right-padded with null bytes.
///
/// ```
/// # use wad::WadString;
/// let wad_str = WadString::new(*b"PLAYPAL\0").unwrap();
/// assert_eq!(wad_str.to_string(), "PLAYPAL");
/// ```
#[derive(Debug, PartialEq)]
pub struct WadString {
	bytes: [u8; 8],
}

impl WadString {
	pub const SIZE_BYTES: usize = 8;

	pub fn new(bytes: [u8; Self::SIZE_BYTES]) -> Result<WadString, ()> {
		// Check for non-ASCII characters
		if bytes.iter().any(|byte| *byte > 127) {
			return Err(());
		}

		Ok(WadString { bytes })
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn wad_header_from_bytes() {
		let mut bytes = [0u8; 12];
		bytes[0..4].clone_from_slice(b"PWAD");
		bytes[4..8].clone_from_slice(&42i32.to_le_bytes()[..]);
		bytes[8..12].clone_from_slice(&1024i32.to_le_bytes()[..]);

		let header = WadHeader::new(bytes).unwrap();

		assert_eq!(header.iwad_or_pwad, WadType::PWAD);
		assert_eq!(header.num_lumps, 42);
		assert_eq!(header.directory_offset_bytes, 1024);
	}

	#[test]
	fn wad_directory_entry_from_bytes() {
		let mut bytes = [0u8; 16];
		bytes[0..4].clone_from_slice(&42i32.to_le_bytes()[..]);
		bytes[4..8].clone_from_slice(&1024i32.to_le_bytes()[..]);
		bytes[8..16].clone_from_slice(b"COLORMAP");

		let dir_entry = WadDirectoryEntry::new(bytes).unwrap();

		assert_eq!(dir_entry.offset_bytes, 42);
		assert_eq!(dir_entry.size_bytes, 1024);
		assert_eq!(dir_entry.lump_name, WadString::new(*b"COLORMAP").unwrap());
	}

	#[test]
	fn wad_type_from_identifier_returns_correct_variant() {
		let result = WadType::new(*b"IWAD");
		assert_eq!(result.unwrap(), WadType::IWAD);

		let result = WadType::new(*b"PWAD");
		assert_eq!(result.unwrap(), WadType::PWAD);

		let result = WadType::new(*b"ZWAD");
		result.unwrap_err(); // Panic on Ok
	}

	#[test]
	fn wad_dir_entry_is_virtual() {
		let nonvirtual_entry = WadDirectoryEntry {
			offset_bytes: 12,
			size_bytes: 10_752,
			lump_name: WadString::new(*b"PLAYPAL\0").unwrap(),
		};
		assert!(!nonvirtual_entry.is_virtual());

		let virtual_entry = WadDirectoryEntry {
			offset_bytes: 0,
			size_bytes: 0,
			lump_name: WadString::new(*b"S_START\0").unwrap(),
		};
		assert!(virtual_entry.is_virtual());
	}

	#[test]
	fn wad_string_new_returns_ok_result() {
		WadString::new(*b"MYSTRING").unwrap();
	}

	#[test]
	fn wad_string_new_returns_err_on_invalid_ascii() {
		WadString::new(*b"INVALID\x80").unwrap_err();
	}

	#[test]
	fn wad_string_display() {
		let wad_str = WadString::new(*b"COLORMAP").unwrap();
		assert_eq!(format!("{}", wad_str), "COLORMAP");

		let wad_str = WadString::new(*b"DEMO1\0\0\0").unwrap();
		assert_eq!(format!("{}", wad_str), "DEMO1");
	}
}
