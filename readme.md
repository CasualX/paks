PAKS file
=========

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/paks.svg)](https://crates.io/crates/paks)
[![docs.rs](https://docs.rs/paks/badge.svg)](https://docs.rs/paks)
[![Build status](https://github.com/CasualX/paks/workflows/CI/badge.svg)](https://github.com/CasualX/paks/actions)

The PAKS file is a light-weight encrypted archive inspired by the Quake PAK format.

Security
--------

This library implements a "Bring Your Own Authenticated Encryption" scheme using [Speck128/128](https://en.wikipedia.org/wiki/Speck_\(cipher\))
in CTR mode with a CBC-MAC. It is not intended for serious cryptographic security.
The encryption and authentication are primarily for obfuscation and integrity checking of game assets in your own projects.
Use at your own risk; this is a fun, experimental format rather than a hardened security solution.

Command-line
------------

This project comes with an executable program `PAKStool` for creating and modifying PAKS files from the command-line.

```
cargo install paks
```

The above command installs the `PAKStool` utility to manipulate PAKS files.

```
PAKStool by Casper - Copyright (c) 2020-2025 Casper <CasualX@users.noreply.github.com>

USAGE
    PAKStool help <COMMAND>
    PAKStool <PAKFILE> <KEY> <COMMAND> [..]

ARGUMENTS
    PAKFILE  Path to a PAKS archive to create or edit.
    KEY      The 128-bit encryption key encoded in hex.
    COMMAND  The subcommand to invoke.

Commands are:
    new      Creates a new empty PAKS archive.
    tree     Displays the directory of the PAKS archive.
    add      Adds a file to the PAKS archive.
    copy     Copies files to the PAKS archive.
    link     Links the file from alternative paths.
    cat      Reads files from the PAKS archive and writes to stdout.
    rm       Removes paths from the PAKS archive.
    mv       Moves files in the PAKS archive.
    gc       Collects garbage left behind by removed files.

    See `PAKStool help <COMMAND>` for more information on a specific command.

EXAMPLES
    PAKStool example.paks 0 new
    PAKStool example.paks 0 add a/b/example < tests/data/example.txt
    PAKStool example.paks 0 link a/b/example aa/bb/example
    PAKStool example.paks 0 tree -u
    PAKStool example.paks 0 rm a/b/example
    PAKStool example.paks 0 cat aa/bb/example
```

Examples
--------

The following code shows how to create a new PAKS file and add some content to it.

Try it out locally: `cargo run --example readme1`.

```rust
// This file contains 65 bytes filled with `0xCF`.
const DATA: &[u8] = &[0xCF; 65];

fn main() {
	let ref key = [13, 42];

	// Create the editor object to create PAKS files in memory.
	let mut edit = paks::MemoryEditor::new();

	// Let's create a file `foo` under a directory `sub`.
	// If a file already exists by this name it will be overwritten.
	edit.create_file(b"sub/foo", DATA, key);

	// When done the editor object can be finalized and returns the encrypted PAKS file as a `Vec<Block>`.
	// It also returns the unencrypted directory for final inspection if desired.
	let (paks, dir) = edit.finish(key);

	// Print the directory.
	print!("The directory:\n\n```\n{}```\n\n", dir.display().to_string());

	// Print the PAKS file itself.
	print!("The RAW data:\n\n```\n{:x?}\n```\n", paks);

	// Create the reader object to inspect PAKS files in memory.
	let read = paks::MemoryReader::from_blocks(paks, key).unwrap();

	// Find the file created earlier and read its data into a `Vec<u8>`.
	let data = read.read(b"sub/foo", key).unwrap();

	// Check that it still matches the expected data.
	assert_eq!(DATA, &data[..]);
}
```

File layout
-----------

The layout of the PAKS file is very simple.

* The header contains a version info number and the location of the directory.

  There is no way to know whether the blob of bytes is a valid PAKS file without the correct key as everything is encrypted by design.

* The data containing the file contents.

  This is an opaque blob of bytes only decodable via information in the directory.

* The directory is a sequence of descriptors encoding a light-weight [TLV structure](https://en.wikipedia.org/wiki/Type-length-value).

  File descriptors contain the location and a cryptographic nonce for accessing the file contents.
  Directory descriptors describe how many of the following descriptors are its children.

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
