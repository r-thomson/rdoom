use super::Lump;
use crate::WadString;
use crate::lump_parser::{LumpParser, Result};

pub use linedefs::LinedefsLump;
pub use sectors::SectorsLump;
pub use sidedefs::SidedefsLump;
pub use things::ThingsLump;
pub use vertexes::VertexesLump;

pub mod things {
	use super::*;

	#[derive(Debug, PartialEq)]
	pub struct ThingsLump {
		pub things: Vec<Thing>,
	}

	impl Lump for ThingsLump {
		fn parse(data: &[u8]) -> Result<Self> {
			let mut parser = LumpParser::new(&data);
			let mut things = Vec::with_capacity(data.len() / linedefs::Linedef::SIZE);

			while parser.has_data_left() {
				things.push(things::Thing {
					x_pos: parser.read_i16()?,
					y_pos: parser.read_i16()?,
					angle: parser.read_i16()?,
					doomed_num: parser.read_i16()?,
					flags: parser.read_i16()?,
				});
			}

			Ok(Self { things })
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct Thing {
		pub x_pos: i16,
		pub y_pos: i16,
		pub angle: i16,
		pub doomed_num: i16,
		pub flags: i16,
	}

	impl Thing {
		pub const SIZE: usize = 10;
	}
}

pub mod linedefs {
	use super::*;

	#[derive(Debug, PartialEq)]
	pub struct LinedefsLump {
		pub linedefs: Vec<linedefs::Linedef>,
	}

	impl Lump for LinedefsLump {
		fn parse(data: &[u8]) -> Result<Self> {
			let mut parser = LumpParser::new(&data);
			let mut linedefs = Vec::with_capacity(data.len() / linedefs::Linedef::SIZE);

			while parser.has_data_left() {
				linedefs.push(linedefs::Linedef {
					vertex_1: parser.read_i16()?,
					vertex_2: parser.read_i16()?,
					flags: parser.read_i16()?,
					special: parser.read_i16()?,
					tag: parser.read_i16()?,
					front_sidedef: parser.read_i16()?,
					back_sidedef: parser.read_i16()?,
				});
			}

			Ok(Self { linedefs })
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct Linedef {
		pub vertex_1: i16,
		pub vertex_2: i16,
		pub flags: i16,
		pub special: i16,
		pub tag: i16,
		pub front_sidedef: i16,
		pub back_sidedef: i16,
	}

	impl Linedef {
		pub const SIZE: usize = 14;
	}
}

pub mod sidedefs {
	use super::*;

	#[derive(Debug, PartialEq)]
	pub struct SidedefsLump {
		pub sidedefs: Vec<sidedefs::Sidedef>,
	}

	impl Lump for SidedefsLump {
		fn parse(data: &[u8]) -> Result<Self> {
			let mut parser = LumpParser::new(&data);
			let mut sidedefs = Vec::with_capacity(data.len() / sidedefs::Sidedef::SIZE);

			while parser.has_data_left() {
				sidedefs.push(sidedefs::Sidedef {
					x_offset: parser.read_i16()?,
					y_offset: parser.read_i16()?,
					upper_tex: WadString::from_bytes(parser.read_chunk()?)?,
					mid_tex: WadString::from_bytes(parser.read_chunk()?)?,
					lower_tex: WadString::from_bytes(parser.read_chunk()?)?,
					sector: parser.read_i16()?,
				});
			}

			Ok(Self { sidedefs })
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct Sidedef {
		pub x_offset: i16,
		pub y_offset: i16,
		pub upper_tex: WadString,
		pub mid_tex: WadString,
		pub lower_tex: WadString,
		pub sector: i16,
	}

	impl Sidedef {
		pub const SIZE: usize = 30;
	}
}

pub mod vertexes {
	use super::*;

	#[derive(Debug, PartialEq)]
	pub struct VertexesLump {
		pub vertexes: Vec<vertexes::Vertex>,
	}

	impl Lump for VertexesLump {
		fn parse(data: &[u8]) -> Result<Self> {
			let mut parser = LumpParser::new(&data);
			let mut vertexes = Vec::with_capacity(data.len() / vertexes::Vertex::SIZE);

			while parser.has_data_left() {
				let x = parser.read_i16()?;
				let y = parser.read_i16()?;

				vertexes.push(vertexes::Vertex { x, y });
			}

			Ok(Self { vertexes })
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct Vertex {
		pub x: i16,
		pub y: i16,
	}

	impl Vertex {
		pub const SIZE: usize = 4;
	}
}

pub mod segs {}

pub mod subsectors {}

pub mod nodes {}

pub mod sectors {
	use super::*;

	#[derive(Debug, PartialEq)]
	pub struct SectorsLump {
		pub sectors: Vec<Sector>,
	}

	impl Lump for SectorsLump {
		fn parse(data: &[u8]) -> Result<Self> {
			let mut parser = LumpParser::new(data);
			let mut sectors = Vec::with_capacity(data.len() / Sector::SIZE);

			while parser.has_data_left() {
				sectors.push(Sector {
					floor_height: parser.read_i16()?,
					ceiling_height: parser.read_i16()?,
					floor_flat: WadString::from_bytes(parser.read_chunk::<8>()?)?,
					ceiling_flat: WadString::from_bytes(parser.read_chunk::<8>()?)?,
					light_level: parser.read_i16()?,
					special: parser.read_i16()?,
					tag: parser.read_i16()?,
				});
			}

			Ok(Self { sectors })
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct Sector {
		pub floor_height: i16,
		pub ceiling_height: i16,
		pub floor_flat: WadString,
		pub ceiling_flat: WadString,
		pub light_level: i16,
		pub special: i16,
		pub tag: i16,
	}

	impl Sector {
		pub const SIZE: usize = 26;
	}
}

pub mod reject {}

pub mod blockmap {}
