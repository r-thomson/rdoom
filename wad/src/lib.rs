/// Where's All the Data?
pub struct Wad {
	pub header: WadHeader,
	pub directory: Vec<WadDirectoryEntry>,
}

pub struct WadHeader {
	pub iwad_or_pwad: WadType,
	pub num_lumps: i32,
	pub directory_offset_bytes: i32,
}

/// Either IWAD or PWAD
#[derive(Debug, Eq, PartialEq)]
pub enum WadType {
	IWAD,
	PWAD,
}

impl TryFrom<&[u8; 4]> for WadType {
	type Error = ();

	fn try_from(value: &[u8; 4]) -> Result<Self, Self::Error> {
		match value {
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
	fn wad_type_from_identifier_returns_correct_variant() {
		let result = WadType::try_from(b"IWAD");
		assert_eq!(result.unwrap(), WadType::IWAD);

		let result = WadType::try_from(b"PWAD");
		assert_eq!(result.unwrap(), WadType::PWAD);

		let result = WadType::try_from(b"ZWAD");
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
