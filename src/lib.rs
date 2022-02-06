//! # time-format
//!
//! This crate does only one thing: format a Unix timestamp.
//!
//! ## Splitting a timestamp into its components
//!
//! The `components_utc()` function returns the components of a timestamp:
//!
//! ```rust
//! let ts = time_format::now().unwrap();
//!
//! let components = time_format::components_utc(ts).unwrap();
//! ```
//!
//! Components are `sec`, `min`, `hour`, `month_day`, `month`, `year`,
//! `week_day` and `year_day`.
//!
//! ## Formatting a timestamp
//!
//! The `strftime_utc()` function formats a timestamp, using the same format as
//! the `strftime()` function of the standard C library.
//!
//! ```rust
//! let ts = time_format::now().unwrap();
//!
//! let s = time_format::strftime_utc("%Y-%m-%d", ts).unwrap();
//! ```
//!
//! ## That's it
//!
//! If you need a minimal crate to get timestamps and perform basic operations on them, check out [coarsetime](https://crates.io/crates/coarsetime).

use std::{
    convert::TryInto,
    ffi::CString,
    fmt,
    mem::MaybeUninit,
    os::raw::{c_char, c_int, c_long},
};

#[allow(non_camel_case_types)]
type time_t = i64;

/// A UNIX timestamp.
pub type TimeStamp = i64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *mut c_char,
}

extern "C" {
    fn gmtime_r(ts: *const time_t, tm: *mut tm) -> *mut tm;
    fn strftime(
        s: *mut c_char,
        maxsize: usize,
        format: *const c_char,
        timeptr: *const tm,
    ) -> usize;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    TimeError,
    FormatError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TimeError => write!(f, "Time error"),
            Error::FormatError => write!(f, "Format error"),
        }
    }
}

impl std::error::Error for Error {}

/// Time components.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Components {
    /// Second.
    pub sec: u8,
    /// Minute.
    pub min: u8,
    /// Hour.
    pub hour: u8,
    /// Day of month.
    pub month_day: u8,
    /// Month - January is 1, December is 12.
    pub month: u8,
    /// Year.
    pub year: i16,
    /// Day of week.
    pub week_day: u8,
    /// Day of year.    
    pub year_day: u16,
}

/// Split a timestamp into its components.
pub fn components_utc(ts_seconds: TimeStamp) -> Result<Components, Error> {
    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { gmtime_r(&ts_seconds, tm.as_mut_ptr() as *mut tm) }.is_null() {
        return Err(Error::TimeError);
    }
    let tm = unsafe { tm.assume_init() };
    Ok(Components {
        sec: tm.tm_sec as _,
        min: tm.tm_min as _,
        hour: tm.tm_hour as _,
        month_day: tm.tm_mday as _,
        month: (1 + tm.tm_mon) as _,
        year: (1900 + tm.tm_year) as _,
        week_day: tm.tm_wday as _,
        year_day: tm.tm_yday as _,
    })
}

/// Return the current UNIX timestamp in seconds.
pub fn now() -> Result<TimeStamp, Error> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::TimeError)?
        .as_secs()
        .try_into()
        .map_err(|_| Error::TimeError)
}

/// Return the current time in the specified format, in the UTC time zone.
/// The time is assumed to be the number of seconds since the Epoch.
pub fn strftime_utc(format: &str, ts_seconds: TimeStamp) -> Result<String, Error> {
    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { gmtime_r(&ts_seconds, tm.as_mut_ptr() as *mut tm) }.is_null() {
        return Err(Error::TimeError);
    }
    let tm = unsafe { tm.assume_init() };

    let format_len = format.len();
    let format = CString::new(format).map_err(|_| Error::FormatError)?;
    let mut buf_size = format_len;
    let mut buf: Vec<u8> = vec![0; buf_size];
    loop {
        let len = unsafe {
            strftime(
                buf.as_mut_ptr() as *mut c_char,
                buf_size,
                format.as_ptr() as *const c_char,
                &tm,
            )
        };
        if len == 0 {
            buf_size *= 2;
            buf.resize(buf_size, 0);
        } else {
            buf.truncate(len);
            return String::from_utf8(buf).map_err(|_| Error::FormatError);
        }
    }
}
