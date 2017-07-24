mod base;
pub use self::base::*;

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use self::win32::*;

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use self::unix::*;

