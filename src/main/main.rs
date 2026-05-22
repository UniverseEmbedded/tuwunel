use std::sync::atomic::Ordering;

use tuwunel::{Server, args, runtime::Runtime};
#[cfg(unix)]
use tuwunel::restart;
use tuwunel_core::{Result, debug_info};

fn main() -> Result {
	let args = args::parse();
	let runtime = Runtime::new(Some(&args))?;
	let server = Server::new(Some(&args), Some(&runtime))?;

	tuwunel::exec(&server, runtime)?;

	#[cfg(unix)]
	if server.server.restarting.load(Ordering::Acquire) {
		restart::restart();
	}

	#[cfg(not(unix))]
	if server.server.restarting.load(Ordering::Acquire) {
		eprintln!("Restart is not supported on this platform (Windows).");
	}

	debug_info!("Exit");
	Ok(())
}
