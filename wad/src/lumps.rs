use std::path::Iter;

pub struct PlaypalLump {
	pub palettes: Vec<playpal::Palette>,
}

impl PlaypalLump {
	pub fn from_bytes(data: &[u8]) -> Result<Self, ()> {
		let (chunks, []) = data.as_chunks::<{ playpal::Palette::SIZE_BYTES }>() else {
			return Err(());
		};

		Ok(PlaypalLump {
			palettes: chunks
				.iter()
				.map(playpal::Palette::from_bytes)
				.collect::<Vec<_>>(),
		})
	}
}

pub mod playpal {
	#[derive(Debug)]
	pub struct Palette {
		pub colors: [Color; Self::NUM_COLORS],
	}

	impl Palette {
		pub const NUM_COLORS: usize = 256;
		pub const SIZE_BYTES: usize = Self::NUM_COLORS * Color::SIZE_BYTES;

		pub fn from_bytes(data: &[u8; Self::SIZE_BYTES]) -> Self {
			let (chunks, []) = data.as_chunks::<{ Color::SIZE_BYTES }>() else {
				unreachable!()
			};

			Self {
				colors: chunks
					.iter()
					.map(Color::from_bytes)
					.collect::<Vec<_>>()
					.try_into()
					.unwrap(),
			}
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct Color {
		pub r: u8,
		pub g: u8,
		pub b: u8,
	}

	impl Color {
		pub const SIZE_BYTES: usize = 3;

		pub fn from_bytes(data: &[u8; Self::SIZE_BYTES]) -> Self {
			Self {
				r: data[0],
				g: data[1],
				b: data[2],
			}
		}
	}

	// Implemented to make testing easier
	impl PartialEq<[u8; 3]> for Color {
		fn eq(&self, other: &[u8; 3]) -> bool {
			[self.r, self.g, self.b] == *other
		}
	}
}

pub struct ColormapLump {
	pub maps: [[u8; playpal::Palette::NUM_COLORS]; Self::NUM_MAPS],
}

impl ColormapLump {
	pub const NUM_MAPS: usize = 34;
	pub const INVULN_INDEX: usize = 32;

	pub fn from_bytes(data: &[u8]) -> Result<Self, ()> {
		if data.len() != playpal::Palette::NUM_COLORS * Self::NUM_MAPS {
			return Err(());
		}

		let (chunks, []) = data.as_chunks::<{ playpal::Palette::NUM_COLORS }>() else {
			unreachable!()
		};

		Ok(Self {
			maps: chunks.try_into().unwrap(),
		})
	}
}

pub struct TexturesLump {
	pub num_textures: i32,
	pub offsets: Vec<i32>,
	pub textures: Vec<textures::TexEntry>,
}

impl TexturesLump {
	pub fn from_bytes(data: &[u8]) -> Result<Self, ()> {
		// TODO: verify the length of the input data is valid
		let num_textures = i32::from_le_bytes(data[0..4].try_into().map_err(|_| ())?);

		let (chunks, []) = data[4..(4 + 4 * num_textures as usize)].as_chunks::<4>() else {
			unreachable!()
		};

		let offsets: Vec<i32> = chunks
			.iter()
			.map(|bytes| i32::from_le_bytes(*bytes))
			.collect();

		let textures: Vec<textures::TexEntry> = offsets
			.iter()
			.map(|offset| textures::TexEntry::from_bytes(&data[(*offset as usize)..]))
			.collect::<Result<_, _>>()
			.map_err(|_| ())?;

		Ok(Self {
			num_textures,
			offsets,
			textures,
		})
	}
}

pub mod textures {
	use crate::wad::WadString;

	pub struct TexEntry {
		pub name: WadString,
		pub _masked: i32,
		pub tex_width: i16,
		pub tex_height: i16,
		pub _columndirectory: i32,
		pub num_patches: i16,
		pub patches: Vec<Patch>,
	}

	impl TexEntry {
		pub fn from_bytes(data: &[u8]) -> Result<Self, ()> {
			let num_patches = i16::from_le_bytes(data[20..22].try_into().map_err(|_| ())?);
			let patches_slice = &data[22..(22 + Patch::SIZE_BYTES * num_patches as usize)];

			let (chunks, []) = patches_slice.as_chunks::<{ Patch::SIZE_BYTES }>() else {
				unreachable!()
			};

			Ok(Self {
				name: WadString::from_bytes(data[0..8].try_into().map_err(|_| ())?)?,
				_masked: i32::from_le_bytes(data[8..12].try_into().map_err(|_| ())?),
				tex_width: i16::from_le_bytes(data[12..14].try_into().map_err(|_| ())?),
				tex_height: i16::from_le_bytes(data[14..16].try_into().map_err(|_| ())?),
				_columndirectory: i32::from_le_bytes(data[16..20].try_into().map_err(|_| ())?),
				num_patches,
				patches: chunks.iter().map(Patch::from_bytes).collect(),
			})
		}
	}

	pub struct Patch {
		pub x_offset: i16,
		pub y_offset: i16,
		pub pname_index: i16,
		pub _stepdir: i16,
		pub _colormap: i16,
	}

	impl Patch {
		const SIZE_BYTES: usize = 10;

		pub fn from_bytes(data: &[u8; Self::SIZE_BYTES]) -> Self {
			Self {
				x_offset: i16::from_le_bytes(data[0..2].try_into().unwrap()),
				y_offset: i16::from_le_bytes(data[2..4].try_into().unwrap()),
				pname_index: i16::from_le_bytes(data[4..6].try_into().unwrap()),
				_stepdir: i16::from_le_bytes(data[6..8].try_into().unwrap()),
				_colormap: i16::from_le_bytes(data[8..10].try_into().unwrap()),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn playpal_color_from_bytes() {
		let color = playpal::Color::from_bytes(&[255, 159, 67]);
		assert_eq!(color.r, 255);
		assert_eq!(color.g, 159);
		assert_eq!(color.b, 67);
	}
}
