# `wad` Package

Handles WAD/lump parsing.

## Architecture

This package is designed around a three-layer architecture:

1. **File** The raw data storage of the WAD file. Could be in-memory or on the disk.
2. **Container** Parsing and representation of the WAD format. Could theoretically be swapped out for other archive formats like PK3.
3. **Lumps** Parsing and representation of Doom engine lumps, e.g. `PLAYPAL`.

## Example

```rust
use std::fs::File;
use wad::*;

let mut file = File::open("DOOM.wad").unwrap();
let wad = Wad::new(&mut file).unwrap();
wad.header.iwad_or_pwad; // WadType::IWAD

// Read a lump to a buffer
let dir_entry = wad
	.directory
	.iter()
	.find(|entry| entry.lump_name == "TEXTURE2")
	.unwrap();
let mut buf = vec![0u8; dir_entry.size_bytes as usize];
dir_entry.read_lump(&mut file, &mut buf).unwrap();

// Parse a lump
let texture2 = wad::lumps::TexturesLump::parse(&buf).unwrap();
let fireblu = texture2
	.textures
	.iter()
	.find(|tex| tex.name == "FIREBLU1")
	.unwrap();
```
