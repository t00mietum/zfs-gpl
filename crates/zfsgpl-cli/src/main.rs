// SPDX-License-Identifier: GPL-2.0-or-later
//
// zgpl: the zfs-gpl command-line front end. Named `zgpl`, not `zpool`/`zfs`,
// so it can sit beside a stock OpenZFS install. Nothing implemented yet.

fn main() {
	let magic = zfsgpl_core::ondisk::UBERBLOCK_MAGIC;
	println!("zgpl (zfs-gpl) - pre-alpha, nothing works yet");
	println!("uberblock magic recognized: {magic:#010x}");
}
