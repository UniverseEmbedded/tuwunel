use crate::Result;

#[cfg(unix)]
use nix::sys::resource::Usage;
#[cfg(unix)]
use nix::sys::resource::{UsageWho, getrusage};

#[cfg(unix)]
use crate::expected;

#[cfg(unix)]
pub fn virt() -> Result<usize> {
	Ok(statm_bytes()?
		.next()
		.expect("incomplete statm contents"))
}

#[cfg(unix)]
pub fn res() -> Result<usize> {
	Ok(statm_bytes()?
		.nth(1)
		.expect("incomplete statm contents"))
}

#[cfg(unix)]
pub fn shm() -> Result<usize> {
	Ok(statm_bytes()?
		.nth(2)
		.expect("incomplete statm contents"))
}

#[cfg(unix)]
pub fn code() -> Result<usize> {
	Ok(statm_bytes()?
		.nth(3)
		.expect("incomplete statm contents"))
}

#[cfg(unix)]
pub fn data() -> Result<usize> {
	Ok(statm_bytes()?
		.nth(5)
		.expect("incomplete statm contents"))
}

#[cfg(unix)]
#[inline]
pub fn statm_bytes() -> Result<impl Iterator<Item = usize>> {
	let page_size = super::page_size()?;

	Ok(statm()?.map(move |pages| expected!(pages * page_size)))
}

#[cfg(target_os = "linux")]
#[inline]
pub fn statm() -> Result<impl Iterator<Item = usize>> {
	use std::{fs::File, io::Read, str};

	use crate::{Error, arrayvec::ArrayVec};

	File::open("/proc/self/statm")
		.map_err(Error::from)
		.and_then(|mut fp| {
			let mut buf = [0; 96];
			let len = fp.read(&mut buf)?;
			let vals = str::from_utf8(&buf[0..len])
				.expect("non-utf8 content in statm")
				.split_ascii_whitespace()
				.map(|val| {
					val.parse()
						.expect("non-integer value in statm contents")
				})
				.collect::<ArrayVec<usize, 12>>();

			Ok(vals.into_iter())
		})
}

#[cfg(not(target_os = "linux"))]
#[inline]
pub fn statm() -> Result<impl Iterator<Item = usize>> { Ok([0, 0, 0, 0, 0, 0].into_iter()) }

#[cfg(unix)]
pub fn usage() -> Result<Usage> { getrusage(UsageWho::RUSAGE_SELF).map_err(Into::into) }

#[cfg(unix)]
#[cfg(any(
	target_os = "linux",
	target_os = "freebsd",
	target_os = "openbsd"
))]
pub fn thread_usage() -> Result<Usage> { getrusage(UsageWho::RUSAGE_THREAD).map_err(Into::into) }

#[cfg(unix)]
#[cfg(not(any(
	target_os = "linux",
	target_os = "freebsd",
	target_os = "openbsd"
)))]
pub fn thread_usage() -> Result<Usage> {
	unimplemented!("RUSAGE_THREAD available on this platform")
}

#[cfg(windows)]
mod windows_usage {
	use crate::Result;

	#[derive(Clone, Debug, Default)]
	pub struct ResourceUsage {
		pub user_time_us: u64,
		pub system_time_us: u64,
		pub max_rss_kb: u64,
		pub shared_mem_kb: u64,
		pub unshared_data_kb: u64,
		pub unshared_stack_kb: u64,
	}

	pub fn usage() -> Result<ResourceUsage> {
		use std::mem::size_of;
		use windows_sys::Win32::Foundation::FILETIME;
		use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
		use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

		let process_handle = unsafe { GetCurrentProcess() };

		let mut creation_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
		let mut exit_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
		let mut kernel_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
		let mut user_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

		unsafe {
			let _ = GetProcessTimes(process_handle, &mut creation_time, &mut exit_time, &mut kernel_time, &mut user_time);
		}

		let user_time_us = (user_time.dwHighDateTime as u64 * 4_294_967_296 + user_time.dwLowDateTime as u64) / 10;
		let system_time_us = (kernel_time.dwHighDateTime as u64 * 4_294_967_296 + kernel_time.dwLowDateTime as u64) / 10;

		let mut mem_counters = PROCESS_MEMORY_COUNTERS {
			cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
			PageFaultCount: 0,
			PeakWorkingSetSize: 0,
			WorkingSetSize: 0,
			QuotaPeakPagedPoolUsage: 0,
			QuotaPagedPoolUsage: 0,
			QuotaPeakNonPagedPoolUsage: 0,
			QuotaNonPagedPoolUsage: 0,
			PagefileUsage: 0,
			PeakPagefileUsage: 0,
		};

		let max_rss_kb: u64 = unsafe {
			if GetProcessMemoryInfo(process_handle, &mut mem_counters, size_of::<PROCESS_MEMORY_COUNTERS>() as u32) != 0 {
				(mem_counters.WorkingSetSize / 1024) as u64
			} else {
				0
			}
		};

		Ok(ResourceUsage {
			user_time_us,
			system_time_us,
			max_rss_kb,
			shared_mem_kb: 0,
			unshared_data_kb: max_rss_kb,
			unshared_stack_kb: max_rss_kb / 2,
		})
	}

	pub fn thread_usage() -> Result<ResourceUsage> {
		use windows_sys::Win32::Foundation::FILETIME;
		use windows_sys::Win32::System::Threading::{GetCurrentThread, GetThreadTimes};

		let thread_handle = unsafe { GetCurrentThread() };

		let mut creation_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
		let mut exit_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
		let mut kernel_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
		let mut user_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

		unsafe {
			let _ = GetThreadTimes(thread_handle, &mut creation_time, &mut exit_time, &mut kernel_time, &mut user_time);
		}

		let user_time_us = (user_time.dwHighDateTime as u64 * 4_294_967_296 + user_time.dwLowDateTime as u64) / 10;
		let system_time_us = (kernel_time.dwHighDateTime as u64 * 4_294_967_296 + kernel_time.dwLowDateTime as u64) / 10;

		Ok(ResourceUsage {
			user_time_us,
			system_time_us,
			max_rss_kb: 0,
			shared_mem_kb: 0,
			unshared_data_kb: 0,
			unshared_stack_kb: 0,
		})
	}
}

#[cfg(windows)]
pub use windows_usage::{usage, thread_usage};

#[cfg(not(any(unix, windows)))]
pub fn usage() -> Result<()> { Ok(()) }

#[cfg(not(any(unix, windows)))]
pub fn thread_usage() -> Result<()> { Ok(()) }
