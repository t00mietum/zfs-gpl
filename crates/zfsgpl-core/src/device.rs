// SPDX-License-Identifier: GPL-2.0-or-later
//
// The block-device seam: positioned reads over a leaf vdev's backing store.
// This is the one place the core reaches std I/O; everything above it works on
// bytes already in memory. Spec context: spec/specs/format/01-vdev-label-uberblock.md.

//! A leaf vdev is any store that answers positioned reads and knows its own
//! usable size - a regular image file or a raw block device alike. `BlockDevice`
//! is that seam; `FileDevice` is the std-backed implementation.

use std::fs::File;
use std::io;
use std::path::Path;

/// What can go wrong reaching the backing store. Structural parse failures are
/// not errors here - a slot that does not hold an uberblock is simply not a
/// candidate (see `vdev`), whereas a failed read is.
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
	#[error("device I/O failed at offset {offset}: {source}")]
	Io {
		offset: u64,
		#[source]
		source: io::Error,
	},
	#[error("read of {len} bytes at offset {offset} runs past the device size {size}")]
	OutOfBounds { offset: u64, len: u64, size: u64 },
}

/// A leaf vdev that answers positioned reads. Sync by design: the core is sync,
/// with async confined to an outer I/O edge if one is ever added.
pub trait BlockDevice {
	/// Usable size in bytes. Label geometry (spec 2.2) is anchored to this.
	fn size(&self) -> u64;

	/// Fill `buf` from `offset`, exactly. Short reads are an error, not a
	/// partial success - a caller asking for a label region wants all of it.
	///
	/// # Errors
	/// [`DeviceError::OutOfBounds`] if the window runs past the device size, or
	/// [`DeviceError::Io`] if the underlying read fails.
	fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<(), DeviceError>;
}

/// A leaf vdev backed by an open file (image or raw device), opened read-only.
#[derive(Debug)]
pub struct FileDevice {
	file: File,
	size: u64,
}

impl FileDevice {
	/// Open `path` read-only and learn its usable size. A regular file reports
	/// its length; a raw block device (whose metadata length reads as zero on
	/// Linux) is measured by seeking to the end, which works for both.
	///
	/// # Errors
	/// Propagates any [`io::Error`] from opening or seeking the path.
	pub fn open(path: impl AsRef<Path>) -> io::Result<FileDevice> {
		use std::io::{Seek, SeekFrom};
		let mut file = File::open(path)?;
		let size = file.seek(SeekFrom::End(0))?;
		Ok(FileDevice { file, size })
	}
}

impl BlockDevice for FileDevice {
	fn size(&self) -> u64 {
		self.size
	}

	fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<(), DeviceError> {
		let len = buf.len() as u64;
		if offset.checked_add(len).is_none_or(|end| end > self.size) {
			return Err(DeviceError::OutOfBounds {
				offset,
				len,
				size: self.size,
			});
		}
		pread_exact(&self.file, offset, buf).map_err(|source| DeviceError::Io { offset, source })
	}
}

/// Positioned exact read, portably. The Unix and Windows positioned-read APIs
/// differ (one fills the buffer, the other may return short and needs a loop),
/// so the platform detail is quarantined here.
#[cfg(unix)]
fn pread_exact(file: &File, offset: u64, buf: &mut [u8]) -> io::Result<()> {
	use std::os::unix::fs::FileExt;
	file.read_exact_at(buf, offset)
}

#[cfg(windows)]
fn pread_exact(file: &File, offset: u64, buf: &mut [u8]) -> io::Result<()> {
	use std::os::windows::fs::FileExt;
	let mut filled = 0usize;
	while filled < buf.len() {
		let n = file.seek_read(&mut buf[filled..], offset + filled as u64)?;
		if n == 0 {
			return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
		}
		filled += n;
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_support::MemDevice;

	#[test]
	fn reads_the_requested_window() {
		let dev = MemDevice {
			bytes: (0..64u8).collect(),
		};
		let mut buf = [0u8; 8];
		dev.read_at(16, &mut buf).unwrap();
		assert_eq!(buf, [16, 17, 18, 19, 20, 21, 22, 23]);
	}

	#[test]
	fn rejects_reads_past_the_end() {
		let dev = MemDevice { bytes: vec![0; 32] };
		let mut buf = [0u8; 8];
		assert!(matches!(
			dev.read_at(28, &mut buf),
			Err(DeviceError::OutOfBounds { .. })
		));
	}
}
