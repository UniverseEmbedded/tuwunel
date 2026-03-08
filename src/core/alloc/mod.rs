//! Integration with allocators

#[cfg(feature = "mimalloc")]
pub mod mi;
#[cfg(feature = "mimalloc")]
pub use mi::{memory_stats, memory_usage, trim};

// jemalloc
#[cfg(all(not(feature = "mimalloc"), not(target_env = "msvc"), feature = "jemalloc"))]
pub mod je;
#[cfg(all(not(feature = "mimalloc"), not(target_env = "msvc"), feature = "jemalloc"))]
pub use je::{memory_stats, memory_usage, trim};

#[cfg(all(
	not(feature = "mimalloc"),
	any(target_env = "msvc", not(feature = "jemalloc"))
))]
pub mod default;
#[cfg(all(
	not(feature = "mimalloc"),
	any(target_env = "msvc", not(feature = "jemalloc"))
))]
pub use default::{memory_stats, memory_usage, trim};
