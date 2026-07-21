// SPDX-License-Identifier: GPL-2.0-or-later
//
// zgpl: the zfs-gpl command-line front end. Named `zgpl`, not `zpool`/`zfs`,
// so it can sit beside a stock OpenZFS install. Read-only, pre-alpha.

use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use zfsgpl_core::device::FileDevice;
use zfsgpl_core::vdev;

fn main() -> ExitCode {
	let args: Vec<String> = std::env::args().skip(1).collect();
	match run(&args) {
		Ok(()) => ExitCode::SUCCESS,
		Err(err) => {
			eprintln!("zgpl: {err:#}");
			ExitCode::FAILURE
		}
	}
}

fn run(args: &[String]) -> Result<()> {
	match args.first().map(String::as_str) {
		Some("scan") => scan(args.get(1)),
		Some(other) => bail!("unknown command '{other}' (try: scan <device|image>)"),
		None => {
			usage();
			Ok(())
		}
	}
}

fn usage() {
	println!("zgpl (zfs-gpl) - pre-alpha, read-only");
	println!("usage: zgpl scan <device|image>   scan a leaf vdev's labels for uberblocks");
}

/// Scan one leaf device/image and report the uberblock candidates found and the
/// active one. This is a discovery view; the self-checksum gate is not yet wired
/// (see `vdev`), so every structurally valid slot is listed.
fn scan(path: Option<&String>) -> Result<()> {
	let Some(path) = path else {
		bail!("scan needs a path: zgpl scan <device|image>");
	};
	let device = FileDevice::open(path).with_context(|| format!("opening {path}"))?;
	let result = vdev::scan(&device).with_context(|| format!("scanning {path}"))?;

	println!(
		"{path}: {} uberblock candidate(s) across the label ring(s)",
		result.candidates.len()
	);
	match result.active() {
		Some(active) => println!(
			"active: label L{} slot {} @ device offset {:#x} - txg {}, {:?} byte order",
			active.label_idx,
			active.slot_index,
			active.device_offset,
			active.uberblock.txg,
			active.uberblock.byte_order,
		),
		None => println!("active: none (no valid uberblock found)"),
	}
	Ok(())
}
