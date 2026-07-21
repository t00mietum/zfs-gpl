// SPDX-License-Identifier: GPL-2.0-or-later
//
// Test-only helpers shared across the core's unit tests. Never compiled into a
// release build.

use crate::device::{BlockDevice, DeviceError};

/// An in-memory `BlockDevice`, so I/O-shaped logic is exercised without a real
/// file or device.
pub(crate) struct MemDevice {
	pub bytes: Vec<u8>,
}

impl BlockDevice for MemDevice {
	fn size(&self) -> u64 {
		self.bytes.len() as u64
	}

	fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<(), DeviceError> {
		let len = buf.len() as u64;
		let Some(end) = offset.checked_add(len).filter(|&end| end <= self.size()) else {
			return Err(DeviceError::OutOfBounds {
				offset,
				len,
				size: self.size(),
			});
		};
		let start = usize::try_from(offset).expect("offset fits usize");
		let end = usize::try_from(end).expect("end fits usize");
		buf.copy_from_slice(&self.bytes[start..end]);
		Ok(())
	}
}
