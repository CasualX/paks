use super::*;

/// Memory reader.
///
/// This implementation keeps the entire PAKS file in memory.
pub struct MemoryReader {
	blocks: Vec<Block>,
	directory: Directory,
}

impl MemoryReader {
	/// Parses the bytes as the PAKS file format for reading.
	///
	/// # Notes
	///
	/// The reader has specific alignment requirements for the buffer.
	/// For this reason the entire byte array will be copied to an internal buffer.
	///
	/// # Errors
	///
	/// * [`ErrorKind::InvalidInput`]: Bytes length is not a multiple of the block size.
	/// * [`ErrorKind::InvalidData`]: Incorrect version info or authentication checks failed.
	pub fn from_bytes(bytes: &[u8], key: &Key) -> Result<MemoryReader, ErrorKind> {
		// The input bytes must be a multiple of the BLOCK_SIZE or this is nonsense
		if bytes.len() % BLOCK_SIZE != 0 {
			return Err(ErrorKind::InvalidInput);
		}

		// Allocate enough space to hold the blocks equivalent
		// This is necessary as internal operations have alignment requirements
		// Copy the input into these blocks
		let mut blocks = vec![Block::default(); bytes.len() / BLOCK_SIZE];
		dataview::bytes_mut(blocks.as_mut_slice())[..bytes.len()].copy_from_slice(bytes);

		match from_blocks(blocks, key) {
			Ok((blocks, directory)) => Ok(MemoryReader { blocks, directory }),
			Err(_) => return Err(ErrorKind::InvalidData),
		}
	}

	/// Parses the blocks as the PAKS file format for reading.
	pub fn from_blocks(blocks: Vec<Block>, key: &Key) -> Result<MemoryReader, Vec<Block>> {
		from_blocks(blocks, key).map(|(blocks, directory)| MemoryReader { blocks, directory })
	}
}

impl ops::Deref for MemoryReader {
	type Target = Directory;
	#[inline]
	fn deref(&self) -> &Directory {
		&self.directory
	}
}

impl MemoryReader {
	/// Reads the contents of a file from the PAKS archive.
	pub fn read(&self, path: &[u8], key: &Key) -> Result<Vec<u8>, ErrorKind> {
		let desc = match self.find_file(path) {
			Some(desc) => desc,
			None => return Err(ErrorKind::NotFound),
		};

		self.read_data(desc, key)
	}

	/// Reads the contents of a file from the PAKS archive into a string.
	pub fn read_to_string(&self, path: &[u8], key: &Key) -> Result<String, ErrorKind> {
		let desc = match self.find_file(path) {
			Some(desc) => desc,
			None => return Err(ErrorKind::NotFound),
		};

		let data = self.read_data(desc, key)?;
		String::from_utf8(data).map_err(|_| ErrorKind::InvalidData)
	}

	/// Decrypts and authenticates the section.
	///
	/// The key is not required to be the same as used to open the PAKS file.
	///
	/// # Errors
	///
	/// * [`ErrorKind::InvalidInput`]: The the descriptor is not a file descriptor.
	/// * [`ErrorKind::InvalidData`]: The file's MAC is incorrect, the file is corrupted.
	#[inline]
	pub fn read_section(&self, section: &Section, key: &Key) -> Result<Vec<Block>, ErrorKind> {
		read_section(&self.blocks, section, key)
	}

	/// Decrypts the contents of the given file descriptor.
	///
	/// See [`read_section`](Self::read_section) for more information.
	#[inline]
	pub fn read_data(&self, desc: &Descriptor, key: &Key) -> Result<Vec<u8>, ErrorKind> {
		read_data(&self.blocks, desc, key)
	}

	/// Decrypts the contents of the given file descriptor into the dest buffer.
	///
	/// See [`read_section`](Self::read_section) for more information.
	#[inline]
	pub fn read_data_into(&self, desc: &Descriptor, key: &Key, byte_offset: usize, dest: &mut [u8]) -> Result<(), ErrorKind> {
		read_data_into(&self.blocks, desc, key, byte_offset, dest)
	}
}
