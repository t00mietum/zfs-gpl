// SPDX-License-Identifier: GPL-2.0-or-later
//
// XDR name/value-list decoder. Spec:
// spec/specs/format/03-config-nvlist-xdr-encoding.md (wire format);
// key catalog is spec 01 sec 3.2-3.6. Pure decode over a byte slice, no I/O.

//! The label config area (label region 3) and every other on-disk nvlist hold a
//! name/value list packed in ZFS's XDR encoding: a 4-byte bootstrap header, then
//! an XDR body of version+flags and a sequence of self-describing pairs ending
//! in a zero terminator. This decodes that byte-for-byte. (spec 03)
//!
//! Two robustness choices, both spec-sanctioned:
//! - Each pair declares its own encoded size (spec 03 sec 3.2 field 1); the
//!   decoder re-syncs the cursor to `pair_start + encoded_size` after each pair
//!   (spec 03 sec 7 step 3d). A value it cannot fully model never desyncs the
//!   stream, and the cursor only ever moves forward - a malformed size errors
//!   rather than looping.
//! - Nesting depth is capped so a corrupt list cannot exhaust the stack.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Deepest nesting a config list is allowed to reach before we treat the input
/// as corrupt. Real configs nest a handful deep (root -> `vdev_tree` -> children
/// -> leaf); this is a generous ceiling that only stops pathological input.
const MAX_DEPTH: u32 = 32;

/// Decode failures. All are "the bytes are not a well-formed XDR nvlist".
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NvError {
	/// Ran off the end of the buffer at the given offset.
	#[error("truncated nvlist at offset {0}")]
	Truncated(usize),
	/// Bootstrap encoding byte was not `1` (XDR); on-disk config is always XDR.
	#[error("nvlist is not XDR-encoded (encoding byte {0})")]
	NotXdr(u8),
	/// A pair's declared encoded size is inconsistent with its contents.
	#[error("inconsistent nvpair size at offset {0}")]
	BadPairSize(usize),
	/// A string (name or value) was not valid UTF-8.
	#[error("non-UTF-8 string in nvlist")]
	BadUtf8,
	/// An nvpair type code outside the known 1..=26 set.
	#[error("unknown nvpair type code {0}")]
	UnknownType(u32),
	/// Nesting exceeded [`MAX_DEPTH`].
	#[error("nvlist nested too deep")]
	TooDeep,
}

type Result<T> = core::result::Result<T, NvError>;

/// A decoded value. One variant per nvpair type code (spec 03 sec 6). Scalars
/// hold the widened Rust type; arrays hold a `Vec`.
#[derive(Debug, Clone, PartialEq)]
pub enum NvValue {
	/// A valueless boolean: presence of the key is the fact (type 1).
	Boolean,
	Byte(u8),
	Int8(i8),
	Uint8(u8),
	Int16(i16),
	Uint16(u16),
	Int32(i32),
	Uint32(u32),
	Int64(i64),
	Uint64(u64),
	/// A `0`/`1` boolean carrying an explicit value (type 21).
	BooleanValue(bool),
	/// High-resolution time, an XDR hyper (type 18).
	HrTime(i64),
	String(String),
	ByteArray(Vec<u8>),
	Int8Array(Vec<i8>),
	Uint8Array(Vec<u8>),
	Int16Array(Vec<i16>),
	Uint16Array(Vec<u16>),
	Int32Array(Vec<i32>),
	Uint32Array(Vec<u32>),
	Int64Array(Vec<i64>),
	Uint64Array(Vec<u64>),
	BooleanArray(Vec<bool>),
	StringArray(Vec<String>),
	NvList(NvList),
	NvListArray(Vec<NvList>),
}

/// One name/value pair.
#[derive(Debug, Clone, PartialEq)]
pub struct NvPair {
	pub name: String,
	pub value: NvValue,
}

/// A decoded name/value list: its header fields plus its pairs in on-disk order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NvList {
	pub version: i32,
	pub flags: u32,
	pub pairs: Vec<NvPair>,
}

impl NvList {
	/// The value for `name`, or `None`. Names are unique in a well-formed list;
	/// the first match wins if a producer ever emitted a duplicate.
	#[must_use]
	pub fn get(&self, name: &str) -> Option<&NvValue> {
		self.pairs.iter().find(|p| p.name == name).map(|p| &p.value)
	}

	/// The `u64` value for `name`, if present and of that type. (spec 01 sec 3.2)
	#[must_use]
	pub fn get_u64(&self, name: &str) -> Option<u64> {
		match self.get(name)? {
			NvValue::Uint64(v) => Some(*v),
			_ => None,
		}
	}

	/// The string value for `name`, if present and of that type. (spec 01 sec 3.2)
	#[must_use]
	pub fn get_str(&self, name: &str) -> Option<&str> {
		match self.get(name)? {
			NvValue::String(s) => Some(s),
			_ => None,
		}
	}

	/// The nested list for `name` (e.g. `vdev_tree`). (spec 01 sec 3.3)
	#[must_use]
	pub fn get_nvlist(&self, name: &str) -> Option<&NvList> {
		match self.get(name)? {
			NvValue::NvList(l) => Some(l),
			_ => None,
		}
	}

	/// The list array for `name` (e.g. `children`). (spec 01 sec 3.3)
	#[must_use]
	pub fn get_nvlist_array(&self, name: &str) -> Option<&[NvList]> {
		match self.get(name)? {
			NvValue::NvListArray(a) => Some(a),
			_ => None,
		}
	}
}

/// Decode a packed XDR nvlist (with its 4-byte bootstrap header) into an
/// [`NvList`]. The label config area (label region 3, spec 01 sec 2.4) is
/// exactly such a buffer; trailing zero padding after the terminator is ignored.
///
/// # Errors
/// Returns [`NvError`] if the bootstrap header is not XDR, the stream is
/// truncated or malformed, a string is not UTF-8, or nesting is too deep.
pub fn decode(bytes: &[u8]) -> Result<NvList> {
	let mut cur = Cursor::new(bytes);
	let header = cur.take(4)?;
	// byte 0 = encoding (1 = XDR), byte 1 = writer endian, bytes 2..3 reserved.
	// XDR is inherently big-endian so the endian byte is not consulted here.
	if header[0] != 1 {
		return Err(NvError::NotXdr(header[0]));
	}
	decode_list(&mut cur, 0)
}

/// Decode one list body: the version+flags header, then pairs to the terminator.
/// Embedded lists (`NVLIST` / `NVLIST_ARRAY` values) start here - they carry a
/// header and terminator but no bootstrap. (spec 03 sec 3.1, 5)
fn decode_list(cur: &mut Cursor, depth: u32) -> Result<NvList> {
	if depth >= MAX_DEPTH {
		return Err(NvError::TooDeep);
	}
	let version = cur.i32()?;
	let flags = cur.u32()?;
	let mut pairs = Vec::new();
	loop {
		let pair_start = cur.pos;
		let encoded_size = cur.u32()? as usize;
		let decoded_size = cur.u32()?;
		// Terminator: both size fields zero. Treat a zero encoded size as the end
		// regardless, so a malformed list cannot stall forward progress. (sec 3.3)
		if encoded_size == 0 || decoded_size == 0 {
			break;
		}
		let name = cur.xdr_string()?;
		let type_code = cur.u32()?;
		let nelem = cur.u32()? as usize;
		let value = decode_value(cur, type_code, nelem, depth)?;
		pairs.push(NvPair { name, value });
		// Re-sync to the declared pair boundary. Only ever forward; a size that
		// would move backwards is corrupt. (sec 7 step 3d)
		let boundary = pair_start
			.checked_add(encoded_size)
			.ok_or(NvError::BadPairSize(pair_start))?;
		if boundary < cur.pos {
			return Err(NvError::BadPairSize(pair_start));
		}
		cur.seek(boundary)?;
	}
	Ok(NvList {
		version,
		flags,
		pairs,
	})
}

/// Decode a pair's value given its type code and element count (spec 03 sec 4-6).
/// `nelem` is the count for every array type (sec 4.4-4.6, 5); scalars ignore it.
// Narrowing casts here undo XDR's widening of sub-32-bit types to a 4-byte unit
// (spec 03 sec 4.1); the value provably fits the narrow type.
#[allow(clippy::cast_possible_truncation)]
fn decode_value(cur: &mut Cursor, type_code: u32, nelem: usize, depth: u32) -> Result<NvValue> {
	Ok(match type_code {
		1 => NvValue::Boolean,
		2 => NvValue::Byte(cur.widened_u8()?),
		3 => NvValue::Int16(cur.i32()? as i16),
		4 => NvValue::Uint16(cur.u32()? as u16),
		5 => NvValue::Int32(cur.i32()?),
		6 => NvValue::Uint32(cur.u32()?),
		7 => NvValue::Int64(cur.i64()?),
		8 => NvValue::Uint64(cur.u64()?),
		9 => NvValue::String(cur.xdr_string()?),
		10 => NvValue::ByteArray(cur.opaque(nelem)?.to_vec()),
		11 => NvValue::Int16Array(cur.array(nelem, |c| Ok(c.i32()? as i16))?),
		12 => NvValue::Uint16Array(cur.array(nelem, |c| Ok(c.u32()? as u16))?),
		13 => NvValue::Int32Array(cur.array(nelem, Cursor::i32)?),
		14 => NvValue::Uint32Array(cur.array(nelem, Cursor::u32)?),
		15 => NvValue::Int64Array(cur.array(nelem, Cursor::i64)?),
		16 => NvValue::Uint64Array(cur.array(nelem, Cursor::u64)?),
		17 => NvValue::StringArray(cur.array(nelem, Cursor::xdr_string)?),
		18 => NvValue::HrTime(cur.i64()?),
		19 => NvValue::NvList(decode_list(cur, depth + 1)?),
		20 => NvValue::NvListArray(cur.array(nelem, |c| decode_list(c, depth + 1))?),
		21 => NvValue::BooleanValue(cur.u32()? != 0),
		22 => NvValue::Int8(cur.i32()? as i8),
		23 => NvValue::Uint8(cur.widened_u8()?),
		24 => NvValue::BooleanArray(cur.array(nelem, |c| Ok(c.u32()? != 0))?),
		25 => NvValue::Int8Array(cur.array(nelem, |c| Ok(c.i32()? as i8))?),
		26 => NvValue::Uint8Array(cur.array(nelem, Cursor::widened_u8)?),
		other => return Err(NvError::UnknownType(other)),
	})
}

/// A forward-only big-endian XDR reader over a byte slice.
struct Cursor<'a> {
	buf: &'a [u8],
	pos: usize,
}

impl<'a> Cursor<'a> {
	fn new(buf: &'a [u8]) -> Self {
		Cursor { buf, pos: 0 }
	}

	/// Borrow the next `n` bytes and advance, or error if they run past the end.
	fn take(&mut self, n: usize) -> Result<&'a [u8]> {
		let end = self
			.pos
			.checked_add(n)
			.ok_or(NvError::Truncated(self.pos))?;
		let slice = self
			.buf
			.get(self.pos..end)
			.ok_or(NvError::Truncated(self.pos))?;
		self.pos = end;
		Ok(slice)
	}

	/// Jump forward to an absolute offset (bounds-checked).
	fn seek(&mut self, pos: usize) -> Result<()> {
		if pos > self.buf.len() {
			return Err(NvError::Truncated(pos));
		}
		self.pos = pos;
		Ok(())
	}

	fn u32(&mut self) -> Result<u32> {
		Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
	}

	fn i32(&mut self) -> Result<i32> {
		Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
	}

	fn u64(&mut self) -> Result<u64> {
		Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
	}

	fn i64(&mut self) -> Result<i64> {
		Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
	}

	/// A 1-byte value widened to a 4-byte unit: the value is the low-order byte
	/// of the big-endian unit. (spec 03 sec 4.1)
	fn widened_u8(&mut self) -> Result<u8> {
		Ok((self.u32()? & 0xff) as u8)
	}

	/// Advance over zero padding to the next 4-byte boundary after `len` payload
	/// bytes. (spec 03 sec 4.2)
	fn skip_pad(&mut self, len: usize) -> Result<()> {
		let pad = (4 - (len % 4)) % 4;
		self.take(pad)?;
		Ok(())
	}

	/// An XDR string: 4-byte length, that many bytes, zero pad to 4. (sec 4.3)
	fn xdr_string(&mut self) -> Result<String> {
		let len = self.u32()? as usize;
		let bytes = self.take(len)?;
		let s = core::str::from_utf8(bytes)
			.map_err(|_| NvError::BadUtf8)?
			.to_string();
		self.skip_pad(len)?;
		Ok(s)
	}

	/// An opaque byte array: `len` raw bytes, zero pad to 4. (spec 03 sec 4.4)
	fn opaque(&mut self, len: usize) -> Result<&'a [u8]> {
		let bytes = self.take(len)?;
		self.skip_pad(len)?;
		Ok(bytes)
	}

	/// `count` elements, each read by `read`, collected in order. (sec 4.5, 4.6, 5)
	fn array<T>(
		&mut self,
		count: usize,
		mut read: impl FnMut(&mut Cursor<'a>) -> Result<T>,
	) -> Result<Vec<T>> {
		let mut out = Vec::with_capacity(count.min(4096));
		for _ in 0..count {
			out.push(read(self)?);
		}
		Ok(out)
	}
}

#[cfg(test)]
mod tests {
	// The test encoder does deliberate length/size byte-fiddling.
	#![allow(clippy::cast_possible_truncation, clippy::manual_is_multiple_of)]
	use super::*;

	// An independent big-endian XDR nvlist encoder, so decode is tested against
	// bytes built by separate logic rather than round-tripped through itself.
	#[derive(Default)]
	struct Enc {
		out: Vec<u8>,
	}

	impl Enc {
		fn u32(&mut self, v: u32) {
			self.out.extend_from_slice(&v.to_be_bytes());
		}
		fn i32(&mut self, v: i32) {
			self.out.extend_from_slice(&v.to_be_bytes());
		}
		fn u64(&mut self, v: u64) {
			self.out.extend_from_slice(&v.to_be_bytes());
		}
		fn str_field(&mut self, s: &str) {
			self.u32(s.len() as u32);
			self.out.extend_from_slice(s.as_bytes());
			while self.out.len() % 4 != 0 {
				self.out.push(0);
			}
		}
		// Encode one pair; `body` writes name+type+nelem+value, then we backfill
		// the leading encoded/decoded sizes.
		fn pair(&mut self, body: impl FnOnce(&mut Enc)) {
			let start = self.out.len();
			self.u32(0); // encoded size placeholder
			self.u32(1); // decoded size (nonzero: not a terminator)
			body(self);
			let size = (self.out.len() - start) as u32;
			self.out[start..start + 4].copy_from_slice(&size.to_be_bytes());
		}
		fn uint64(&mut self, name: &str, v: u64) {
			self.pair(|e| {
				e.str_field(name);
				e.u32(8); // UINT64
				e.u32(1); // nelem
				e.u64(v);
			});
		}
		fn string(&mut self, name: &str, v: &str) {
			self.pair(|e| {
				e.str_field(name);
				e.u32(9); // STRING
				e.u32(1);
				e.str_field(v);
			});
		}
		fn uint64_array(&mut self, name: &str, vs: &[u64]) {
			self.pair(|e| {
				e.str_field(name);
				e.u32(16); // UINT64_ARRAY
				e.u32(vs.len() as u32);
				for &v in vs {
					e.u64(v);
				}
			});
		}
		fn boolean(&mut self, name: &str) {
			self.pair(|e| {
				e.str_field(name);
				e.u32(1); // BOOLEAN, valueless
				e.u32(0); // nelem 0
			});
		}
		// Encode a nested value (NVLIST or NVLIST_ARRAY) via a callback that emits
		// the inner list bytes using `list_body`.
		fn nvlist(&mut self, name: &str, inner: impl FnOnce(&mut Enc)) {
			self.pair(|e| {
				e.str_field(name);
				e.u32(19); // NVLIST
				e.u32(1);
				list_body(e, inner);
			});
		}
		fn nvlist_array(&mut self, name: &str, lists: &[fn(&mut Enc)]) {
			self.pair(|e| {
				e.str_field(name);
				e.u32(20); // NVLIST_ARRAY
				e.u32(lists.len() as u32);
				for &inner in lists {
					list_body(e, inner);
				}
			});
		}
		fn terminate(&mut self) {
			self.u32(0);
			self.u32(0);
		}
	}

	// Write a list body: version+flags header, the caller's pairs, terminator.
	fn list_body(e: &mut Enc, pairs: impl FnOnce(&mut Enc)) {
		e.i32(0); // version
		e.u32(1); // flags: unique-by-name
		pairs(e);
		e.terminate();
	}

	// A full packed nvlist: 4-byte bootstrap header + list body.
	fn packed(pairs: impl FnOnce(&mut Enc)) -> Vec<u8> {
		let mut e = Enc::default();
		e.out.extend_from_slice(&[1, 1, 0, 0]); // XDR, little-endian writer
		list_body(&mut e, pairs);
		e.out
	}

	#[test]
	fn hand_crafted_single_uint64_is_byte_exact() {
		// Independently spell out the bytes for one { "txg": 42 } pair and confirm
		// the framing/endianness assumptions, not just self-consistency.
		let bytes = [
			1, 1, 0, 0, // bootstrap: XDR
			0, 0, 0, 0, // version 0
			0, 0, 0, 1, // flags
			0, 0, 0, 32, // pair encoded size (this field through the value)
			0, 0, 0, 1, // decoded size (nonzero)
			0, 0, 0, 3, // name length 3
			b't', b'x', b'g', 0, // "txg" + pad
			0, 0, 0, 8, // type UINT64
			0, 0, 0, 1, // nelem 1
			0, 0, 0, 0, 0, 0, 0, 42, // value 42
			0, 0, 0, 0, 0, 0, 0, 0, // terminator
		];
		let nv = decode(&bytes).unwrap();
		assert_eq!(nv.get_u64("txg"), Some(42));
	}

	#[test]
	fn rejects_non_xdr_bootstrap() {
		let bytes = [0, 1, 0, 0, 0, 0, 0, 0]; // encoding byte 0 = native
		assert_eq!(decode(&bytes), Err(NvError::NotXdr(0)));
	}

	#[test]
	fn decodes_scalars_and_arrays() {
		let bytes = packed(|e| {
			e.uint64("version", 5000);
			e.string("name", "tank");
			e.uint64_array("hole_array", &[1, 4, 9]);
			e.boolean("com.delphix:embedded_data");
		});
		let nv = decode(&bytes).unwrap();
		assert_eq!(nv.get_u64("version"), Some(5000));
		assert_eq!(nv.get_str("name"), Some("tank"));
		assert_eq!(
			nv.get("hole_array"),
			Some(&NvValue::Uint64Array(vec![1, 4, 9]))
		);
		assert_eq!(nv.get("com.delphix:embedded_data"), Some(&NvValue::Boolean));
		assert!(nv.get("absent").is_none());
	}

	#[test]
	fn decodes_nested_vdev_tree_and_children() {
		// Shape mirrors a real label: pool keys + vdev_tree { type, children[] }.
		let bytes = packed(|e| {
			e.uint64("pool_guid", 0x0123_4567_89ab_cdef);
			e.string("name", "tank");
			e.nvlist("vdev_tree", |t| {
				t.string("type", "mirror");
				t.uint64("ashift", 12);
				t.nvlist_array(
					"children",
					&[
						|c: &mut Enc| {
							c.string("type", "disk");
							c.uint64("guid", 111);
						},
						|c: &mut Enc| {
							c.string("type", "disk");
							c.uint64("guid", 222);
						},
					],
				);
			});
		});
		let nv = decode(&bytes).unwrap();
		assert_eq!(nv.get_u64("pool_guid"), Some(0x0123_4567_89ab_cdef));
		let tree = nv.get_nvlist("vdev_tree").unwrap();
		assert_eq!(tree.get_str("type"), Some("mirror"));
		assert_eq!(tree.get_u64("ashift"), Some(12));
		let children = tree.get_nvlist_array("children").unwrap();
		assert_eq!(children.len(), 2);
		assert_eq!(children[0].get_u64("guid"), Some(111));
		assert_eq!(children[1].get_str("type"), Some("disk"));
	}

	#[test]
	fn truncated_input_errors_not_panics() {
		let mut bytes = packed(|e| e.uint64("version", 1));
		bytes.truncate(bytes.len() - 6); // chop into the value
		assert!(matches!(decode(&bytes), Err(NvError::Truncated(_))));
	}

	#[test]
	fn trailing_zero_padding_is_ignored() {
		// The label nvlist region is zero-filled after the terminator (spec 01
		// sec 2.5); decode must stop at the terminator, not choke on the padding.
		let mut bytes = packed(|e| e.uint64("txg", 7));
		bytes.resize(bytes.len() + 4096, 0);
		let nv = decode(&bytes).unwrap();
		assert_eq!(nv.get_u64("txg"), Some(7));
	}
}
