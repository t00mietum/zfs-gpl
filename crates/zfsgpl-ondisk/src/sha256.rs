// SPDX-License-Identifier: GPL-2.0-or-later
//
// SHA-256 (FIPS 180-4), the label/uberblock self-checksum algorithm.
// Spec: spec/specs/format/02-label-checksum-sha256-packing.md sec 1;
// the algorithm itself is the public FIPS 180-4 standard, not ZFS-specific.

//! A from-scratch SHA-256 so the on-disk crate stays dependency-free and
//! `alloc`-free. Streaming so a caller can hash a region plus an injected
//! verifier without copying the region (spec 02 needs exactly that). Verified
//! against the published FIPS test vectors the spec cites.

/// Streaming SHA-256 state.
#[derive(Clone)]
pub struct Sha256 {
	hash: [u32; 8],
	block: [u8; 64],
	filled: usize,
	total_len: u64,
}

// Round constants: first 32 bits of the fractional parts of the cube roots of
// the first 64 primes. (FIPS 180-4 sec 4.2.2). Kept in the canonical
// eight-hex-digit form the standard tabulates, so no digit separators.
#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
const K: [u32; 64] = [
	0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
	0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
	0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
	0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
	0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
	0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
	0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
	0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// Initial hash: first 32 bits of the fractional parts of the square roots of
// the first 8 primes. (FIPS 180-4 sec 5.3.3)
const H0: [u32; 8] = [
	0x6a09_e667,
	0xbb67_ae85,
	0x3c6e_f372,
	0xa54f_f53a,
	0x510e_527f,
	0x9b05_688c,
	0x1f83_d9ab,
	0x5be0_cd19,
];

impl Sha256 {
	#[must_use]
	pub fn new() -> Self {
		Sha256 {
			hash: H0,
			block: [0u8; 64],
			filled: 0,
			total_len: 0,
		}
	}

	/// Absorb more input. May be called any number of times.
	pub fn update(&mut self, mut data: &[u8]) {
		self.total_len = self.total_len.wrapping_add(data.len() as u64);
		// Top off a partially filled block first.
		if self.filled > 0 {
			let need = 64 - self.filled;
			let take = need.min(data.len());
			self.block[self.filled..self.filled + take].copy_from_slice(&data[..take]);
			self.filled += take;
			data = &data[take..];
			if self.filled == 64 {
				let block = self.block;
				self.compress(&block);
				self.filled = 0;
			} else {
				// Ran out of input without filling the block; the partial block
				// (and its length) must stand until the next update.
				return;
			}
		}
		// Compress whole blocks straight out of the input.
		let mut chunks = data.chunks_exact(64);
		for chunk in &mut chunks {
			let mut block = [0u8; 64];
			block.copy_from_slice(chunk);
			self.compress(&block);
		}
		// Stash the remainder.
		let rest = chunks.remainder();
		self.block[..rest.len()].copy_from_slice(rest);
		self.filled = rest.len();
	}

	/// Finish and return the 32-byte digest D[0..31] (H0 first, big-endian).
	#[must_use]
	pub fn finalize(mut self) -> [u8; 32] {
		let bit_len = self.total_len.wrapping_mul(8);
		// Pad: 0x80, then zeros, then the 64-bit big-endian length.
		self.update(&[0x80]);
		while self.filled != 56 {
			self.update(&[0x00]);
		}
		self.update(&bit_len.to_be_bytes());
		debug_assert_eq!(self.filled, 0);

		let mut out = [0u8; 32];
		for (word, chunk) in self.hash.iter().zip(out.chunks_exact_mut(4)) {
			chunk.copy_from_slice(&word.to_be_bytes());
		}
		out
	}

	// a..h are the eight working variables named exactly as in FIPS 180-4 sec 6.2.
	#[allow(clippy::many_single_char_names)]
	fn compress(&mut self, block: &[u8; 64]) {
		let mut w = [0u32; 64];
		for (word, chunk) in w.iter_mut().zip(block.chunks_exact(4)) {
			*word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
		}
		for i in 16..64 {
			let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
			let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
			w[i] = w[i - 16]
				.wrapping_add(s0)
				.wrapping_add(w[i - 7])
				.wrapping_add(s1);
		}

		let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.hash;
		for i in 0..64 {
			let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
			let ch = (e & f) ^ ((!e) & g);
			let t1 = h
				.wrapping_add(s1)
				.wrapping_add(ch)
				.wrapping_add(K[i])
				.wrapping_add(w[i]);
			let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
			let maj = (a & b) ^ (a & c) ^ (b & c);
			let t2 = s0.wrapping_add(maj);
			h = g;
			g = f;
			f = e;
			e = d.wrapping_add(t1);
			d = c;
			c = b;
			b = a;
			a = t1.wrapping_add(t2);
		}
		for (slot, v) in self.hash.iter_mut().zip([a, b, c, d, e, f, g, h]) {
			*slot = slot.wrapping_add(v);
		}
	}
}

impl Default for Sha256 {
	fn default() -> Self {
		Self::new()
	}
}

/// One-shot SHA-256 of a byte slice.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
	let mut hasher = Sha256::new();
	hasher.update(data);
	hasher.finalize()
}

#[cfg(test)]
mod tests {
	use super::*;

	fn hex(bytes: &[u8]) -> String {
		use std::fmt::Write;
		bytes.iter().fold(String::new(), |mut s, b| {
			write!(s, "{b:02x}").unwrap();
			s
		})
	}

	// The two FIPS vectors spec 02 sec 5 cites for the packing test.
	#[test]
	fn fips_empty_and_abc() {
		assert_eq!(
			hex(&sha256(b"")),
			"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
		);
		assert_eq!(
			hex(&sha256(b"abc")),
			"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
		);
	}

	// Multi-block input that crosses the 56-byte pad boundary (>64 bytes).
	#[test]
	fn fips_two_block() {
		let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
		assert_eq!(
			hex(&sha256(input)),
			"248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
		);
	}

	// Streaming in odd-sized chunks must match the one-shot digest.
	#[test]
	#[allow(clippy::cast_possible_truncation)]
	fn streaming_matches_oneshot() {
		let data: Vec<u8> = (0..1000u32).map(|i| (i % 251) as u8).collect();
		let oneshot = sha256(&data);
		let mut hasher = Sha256::new();
		for chunk in data.chunks(7) {
			hasher.update(chunk);
		}
		assert_eq!(hasher.finalize(), oneshot);
	}
}
