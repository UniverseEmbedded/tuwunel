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

#[cfg(unix)]
use nix::sys::resource::Usage as NixUsage;

#[cfg(unix)]
use nix::sys::resource::{UsageWho, getrusage};

#[cfg(unix)]
impl From<NixUsage> for ResourceUsage {
	fn from(nix: NixUsage) -> Self {
		ResourceUsage {
			user_time_us: nix.user_time().as_micros() as u64,
			system_time_us: nix.system_time().as_micros() as u64,
			max_rss_kb: nix.max_rss() as u64,
			shared_mem_kb: 0,
			unshared_data_kb: 0,
			unshared_stack_kb: 0,
		}
	}
}

#[cfg(unix)]
pub fn usage() -> Result<ResourceUsage> {
	getrusage(UsageWho::RUSAGE_SELF).map(ResourceUsage::from).map_err(Into::into)
}

#[cfg(unix)]
pub fn thread_usage() -> Result<ResourceUsage> {
	getrusage(UsageWho::RUSAGE_THREAD).map(ResourceUsage::from).map_err(Into::into)
}

#[cfg(windows)]
pub fn usage() -> Result<ResourceUsage> {
	use std::mem::size_of;
	use windows_sys::Win32::Foundation::FILETIME;
	use windows_sys::Win32::System::Threading::{
		GetProcessTimes, GetCurrentProcess,
	};
	use windows_sys::Win32::System::ProcessStatus::GetProcessMemoryInfo;
	use windows_sys::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS;

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

#[cfg(windows)]
pub fn thread_usage() -> Result<ResourceUsage> {
	use windows_sys::Win32::Foundation::FILETIME;
	use windows_sys::Win32::System::Threading::{
		GetThreadTimes, GetCurrentThread,
	};

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

#[cfg(not(any(unix, windows)))]
pub fn usage() -> Result<ResourceUsage> {
	Ok(ResourceUsage::default())
}

#[cfg(not(any(unix, windows)))]
pub fn thread_usage() -> Result<ResourceUsage> {
	Ok(ResourceUsage::default())
}
