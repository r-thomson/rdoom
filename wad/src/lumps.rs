use crate::lump_parser::{LumpParser, Result};
use crate::WadString;

pub struct PlaypalLump {
	pub palettes: Vec<playpal::Palette>,
}

impl PlaypalLump {
	pub fn parse(data: &[u8]) -> Result<Self> {
		let mut parser = LumpParser::new(&data);

		let mut palettes = Vec::with_capacity(data.len() / playpal::Palette::BYTES);
		while parser.has_data_left() {
			let bytes = parser.read_chunk::<{ playpal::Palette::BYTES }>()?;
			palettes.push(playpal::Palette::from_bytes(&bytes));
		}

		// has_data_left() must be false, so no need to finish()

		Ok(PlaypalLump { palettes })
	}
}

pub mod playpal {
	#[derive(Debug)]
	pub struct Palette {
		pub colors: [Color; Self::NUM_COLORS],
	}

	impl Palette {
		pub const NUM_COLORS: usize = 256;
		pub const BYTES: usize = Self::NUM_COLORS * Color::BYTES;

		pub fn from_bytes(bytes: &[u8; Self::BYTES]) -> Self {
			let (chunks, _) = bytes.as_chunks::<3>();

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
		pub const BYTES: usize = 3;

		pub fn from_bytes(data: &[u8; Self::BYTES]) -> Self {
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

	pub fn parse(data: &[u8]) -> Result<Self> {
		let mut parser = LumpParser::new(&data);

		let maps: Vec<_> = (0..Self::NUM_MAPS)
			.map(|_| parser.read_chunk::<{ playpal::Palette::NUM_COLORS }>())
			.collect::<Result<_>>()?;

		parser.finish()?;

		Ok(Self {
			maps: maps.try_into().unwrap(),
		})
	}
}

/// The TEXTURE1 and TEXTURE2 lumps.
pub struct TexturesLump {
	pub num_textures: i32,
	pub offsets: Vec<i32>,
	pub textures: Vec<textures::TexEntry>,
}

impl TexturesLump {
	pub fn parse(data: &[u8]) -> Result<Self> {
		let mut parser = LumpParser::new(&data);

		let num_textures = parser.read_i32()?;

		let offsets: Vec<i32> = (0..num_textures)
			.map(|_| parser.read_i32())
			.collect::<Result<_>>()?;

		// TODO: Evaluate the best parsing strategy here. There are a few ways you
		//  could go about this, depending on the assumptions you make about WADs.
		//  Mainly, are the TexEntries always contiguous in the lump?
		let textures: Vec<textures::TexEntry> = offsets
			.iter()
			.map(|offset| textures::TexEntry::parse(&data[(*offset as usize)..]))
			.collect::<Result<_>>()?;

		Ok(Self {
			num_textures,
			offsets,
			textures,
		})
	}
}

pub mod textures {
	use super::{LumpParser, Result, WadString};

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
		pub fn parse(data: &[u8]) -> Result<Self> {
			let mut parser = LumpParser::new(&data);

			let name = WadString::from_bytes(parser.read_chunk::<8>()?)?;
			let _masked = parser.read_i32()?;
			let tex_width = parser.read_i16()?;
			let tex_height = parser.read_i16()?;
			let _columndirectory = parser.read_i32()?;
			let num_patches = parser.read_i16()?;

			let patches: Vec<Patch> = (0..num_patches)
				.map(|_| {
					let bytes = parser.read_chunk::<10>()?;
					Ok(Patch::from_bytes(&bytes))
				})
				.collect::<Result<_>>()?;

			// For now we intentionally don't call finish()

			Ok(Self {
				name,
				_masked,
				tex_width,
				tex_height,
				_columndirectory,
				num_patches,
				patches,
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
		const NUM_BYTES: usize = 10;

		pub fn from_bytes(bytes: &[u8; Self::NUM_BYTES]) -> Self {
			Self {
				x_offset: i16::from_le_bytes(bytes[0..2].try_into().unwrap()),
				y_offset: i16::from_le_bytes(bytes[2..4].try_into().unwrap()),
				pname_index: i16::from_le_bytes(bytes[4..6].try_into().unwrap()),
				_stepdir: i16::from_le_bytes(bytes[6..8].try_into().unwrap()),
				_colormap: i16::from_le_bytes(bytes[8..10].try_into().unwrap()),
			}
		}
	}
}

pub struct PnamesLump {
	pub pnames: Vec<WadString>,
}

impl PnamesLump {
	pub fn parse(data: &[u8]) -> Result<Self> {
		let mut parser = LumpParser::new(&data);

		let num_patches = parser.read_i32()?;

		let pnames: Vec<WadString> = (0..num_patches)
			.map(|_| WadString::from_bytes(parser.read_chunk::<8>()?))
			.collect::<Result<_>>()?;

		parser.finish()?;

		Ok(Self { pnames })
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
