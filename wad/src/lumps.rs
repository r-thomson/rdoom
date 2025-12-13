use crate::lump_parser::Result;

pub trait Lump: Sized {
	fn parse(data: &[u8]) -> Result<Self>;
}

mod gfx;
mod map;

pub use gfx::*;
pub use map::*;
