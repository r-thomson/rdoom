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
