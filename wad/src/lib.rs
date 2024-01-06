use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

/// Where's All the Data?
pub struct Wad {
	file: File,
	pub header: WadHeader,
	pub directory: Vec<WadDirectoryEntry>,
}

impl Wad {
	pub fn new(mut file: File) -> Result<Wad, ()> {
		let Ok(_) = file.seek(SeekFrom::Start(0)) else {
			return Err(());
		};

		let mut header_buf = [0; 12];
		let Ok(_) = file.read_exact(&mut header_buf) else {
			return Err(());
		};

		let Ok(header) = WadHeader::new(header_buf) else {
			return Err(());
		};

		let directory = vec![];

		Ok(Wad {
			file,
			header,
			directory,
		})
	}
}

pub struct WadHeader {
	pub iwad_or_pwad: WadType,
	pub num_lumps: i32,
	pub directory_offset_bytes: i32,
}

impl WadHeader {
	fn new(data: [u8; 12]) -> Result<WadHeader, ()> {
		Ok(WadHeader {
			iwad_or_pwad: WadType::try_from(<[u8; 4]>::try_from(&data[0..4]).unwrap())?,
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

impl TryFrom<[u8; 4]> for WadType {
	type Error = ();

	fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
		match &value {
			b"IWAD" => Ok(Self::IWAD),
			b"PWAD" => Ok(Self::PWAD),
			_ => Err(()),
		}
	}
}

pub struct WadDirectoryEntry {
	pub offset_bytes: i32,
	pub size_bytes: i32,
	pub lump_name: [u8; 8],
}

impl WadDirectoryEntry {
	/// Virtual lumps have a size of zero and only appear in the directory
	pub fn is_virtual(&self) -> bool {
		self.size_bytes == 0
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
	fn wad_type_from_identifier_returns_correct_variant() {
		let result = WadType::try_from(*b"IWAD");
		assert_eq!(result.unwrap(), WadType::IWAD);

		let result = WadType::try_from(*b"PWAD");
		assert_eq!(result.unwrap(), WadType::PWAD);

		let result = WadType::try_from(*b"ZWAD");
		result.unwrap_err(); // Panic on Ok
	}

	#[test]
	fn wad_dir_entry_is_virtual() {
		let nonvirtual_entry = WadDirectoryEntry {
			offset_bytes: 12,
			size_bytes: 10_752,
			lump_name: b"PLAYPAL\0".to_owned(),
		};
		assert!(!nonvirtual_entry.is_virtual());

		let virtual_entry = WadDirectoryEntry {
			offset_bytes: 0,
			size_bytes: 0,
			lump_name: b"S_START\0".to_owned(),
		};
		assert!(virtual_entry.is_virtual());
	}
}
