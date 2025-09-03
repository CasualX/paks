
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
	print!("The directory:\n\n```\n{}```\n\n", dir.display());

	// Print the PAKS file itself.
	print!("The RAW data:\n\n```\n{:x?}\n```\n", paks);

	// Create the reader object to inspect PAKS files in memory.
	let read = paks::MemoryReader::from_blocks(paks, key).unwrap();

	// Find the file created earlier and read its data into a `Vec<u8>`.
	let data = read.read(b"sub/foo", key).unwrap();

	// Check that it still matches the expected data.
	assert_eq!(DATA, &data[..]);
}
