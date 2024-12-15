/*!
# FileIO based PAKS file format implementation

Implements the PAKS file format using [`std::fs::File`].
*/

use std::{fs, path::Path, io, io::prelude::*};
use super::*;

/// Reads a PAKS file from a stream.
///
/// This method reads and decrypts the PAKS file header.
/// If the header is invalid or its MAC check fails, [`io::ErrorKind::InvalidData`] is returned.
///
/// Then it reads all the blocks in the PAKS file as specified by the directory.
pub fn read<F: Read>(mut file: F, key: &Key) -> io::Result<Vec<Block>> {
	// Read the header
	let mut header: Header = dataview::zeroed();
	file.read_exact(dataview::bytes_mut(&mut header))?;

	// Set the encrypted header aside
	let header2 = header;

	// Decrypt and validate the header
	if !crypt::decrypt_header(&mut header, key) {
		return Err(io::Error::from(io::ErrorKind::InvalidData));
	}

	// Use information from the header to calculate the total size of the PAKS file
	// This code assumes the directory is the very last thing in the PAKS file
	let blocks_len = usize::max(Header::BLOCKS_LEN, header.info.directory.offset as usize + header.info.directory.size as usize * Descriptor::BLOCKS_LEN);
	let mut blocks = vec![Block::default(); blocks_len];

	// Copy the encrypted header into the output since it's already read from the file
	blocks[..Header::BLOCKS_LEN].copy_from_slice(header2.as_ref());

	// Then read the rest of the PAKS file
	file.read_exact(dataview::bytes_mut(&mut blocks[Header::BLOCKS_LEN..]))?;

	Ok(blocks)
}

#[inline(always)]
fn read_header(file: &mut fs::File, key: &Key) -> io::Result<(InfoHeader, Directory)> {
	// Read the header
	let mut header: Header = dataview::zeroed();
	file.read_exact(dataview::bytes_mut(&mut header))?;

	// Decrypt the header and validate
	if !crypt::decrypt_header(&mut header, key) {
		Err(io::ErrorKind::InvalidData)?;
	}

	// Read the directory
	file.seek(io::SeekFrom::Start(header.info.directory.offset as u64 * BLOCK_SIZE as u64))?;
	let mut directory = Directory::from(vec![Descriptor::default(); header.info.directory.size as usize]);
	file.read_exact(dataview::bytes_mut(directory.as_mut()))?;

	// Decrypt the directory
	if !crypt::decrypt_section(directory.as_blocks_mut(), &header.info.directory, key) {
		Err(io::ErrorKind::InvalidData)?;
	}

	Ok((header.info, directory))
}

fn read_section(mut file: &fs::File, section: &Section, key: &Key) -> io::Result<Vec<Block>> {
	// Read the data to memory buffer
	let file_offset = section.offset as u64 * BLOCK_SIZE as u64;
	file.seek(io::SeekFrom::Start(file_offset))?;
	let mut blocks = vec![Block::default(); section.size as usize];
	file.read_exact(dataview::bytes_mut(blocks.as_mut_slice()))?;

	// Decrypt the data inplace
	if !crypt::decrypt_section(&mut blocks, section, key) {
		Err(io::ErrorKind::InvalidData)?;
	}

	Ok(blocks)
}

fn read_data(file: &fs::File, desc: &Descriptor, key: &Key) -> io::Result<Vec<u8>> {
	if !desc.is_file() {
		Err(io::ErrorKind::InvalidInput)?;
	}

	let blocks = read_section(file, &desc.section, key)?;

	// Figure out which part of the blocks to copy
	let data = dataview::bytes(blocks.as_slice());
	let len = usize::min(data.len(), desc.content_size as usize);
	Ok(data[..len].to_vec())
}

fn read_data_into(file: &fs::File, desc: &Descriptor, key: &Key, byte_offset: usize, dest: &mut [u8]) -> io::Result<()> {
	if !desc.is_file() {
		Err(io::ErrorKind::InvalidInput)?;
	}

	let blocks = read_section(file, &desc.section, key)?;

	// Figure out which part of the blocks to copy
	let data = match dataview::bytes(blocks.as_slice()).get(byte_offset..byte_offset + dest.len()) {
		Some(data) => data,
		None => Err(io::ErrorKind::InvalidInput)?,
	};

	// Copy the data to its destination
	dest.copy_from_slice(data);

	Ok(())
}

mod reader;
mod editor;
mod edit_file;

pub use self::reader::FileReader;
pub use self::editor::FileEditor;
pub use self::edit_file::FileEditFile;

#[cfg(test)]
mod tests;
