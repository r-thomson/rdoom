# `wad` Package

Handles WAD/lump parsing.

## Architecture

This package is designed around a three-layer architecture:

1. **File** The raw data storage of the WAD file. Could be in-memory or on the disk.
2. **Container** Parsing and representation of the WAD format. Could theoretically be swapped out for other archive formats like PK3.
3. **Lumps** Parsing and representation of Doom engine lumps, e.g. `PLAYPAL`.
