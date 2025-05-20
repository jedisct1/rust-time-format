#![doc = include_str!("../README.md")]

use std::{
    convert::TryInto,
    ffi::CString,
    fmt,
    mem::MaybeUninit,
    os::raw::{c_char, c_int, c_long},
};

#[allow(non_camel_case_types)]
type time_t = i64;

/// A UNIX timestamp in seconds.
pub type TimeStamp = i64;

/// A UNIX timestamp with millisecond precision.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TimeStampMs {
    /// Seconds since the UNIX epoch.
    pub seconds: i64,
    /// Milliseconds component (0-999).
    pub milliseconds: u16,
}

impl TimeStampMs {
    /// Create a new TimeStampMs from seconds and milliseconds.
    pub fn new(seconds: i64, milliseconds: u16) -> Self {
        let milliseconds = milliseconds % 1000;
        Self {
            seconds,
            milliseconds,
        }
    }

    /// Convert from a TimeStamp (seconds only).
    pub fn from_timestamp(ts: TimeStamp) -> Self {
        Self {
            seconds: ts,
            milliseconds: 0,
        }
    }

    /// Get the total milliseconds since the UNIX epoch.
    pub fn total_milliseconds(&self) -> i64 {
        self.seconds * 1000 + self.milliseconds as i64
    }
}

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
    fn localtime_r(ts: *const time_t, tm: *mut tm) -> *mut tm;
    fn strftime(s: *mut c_char, maxsize: usize, format: *const c_char, timeptr: *const tm)
        -> usize;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    /// Error occurred while parsing or converting time
    TimeError,
    /// Error occurred with timestamp value (e.g., timestamp out of range)
    InvalidTimestamp,
    /// Error occurred while formatting time
    FormatError,
    /// Error with format string (e.g., invalid format specifier)
    InvalidFormatString,
    /// Error with UTF-8 conversion from C string
    Utf8Error,
    /// Error with null bytes in input strings
    NullByteError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TimeError => write!(f, "Time processing error"),
            Error::InvalidTimestamp => write!(f, "Invalid timestamp value"),
            Error::FormatError => write!(f, "Time formatting error"),
            Error::InvalidFormatString => write!(f, "Invalid format string"),
            Error::Utf8Error => write!(f, "UTF-8 conversion error"),
            Error::NullByteError => write!(f, "String contains null bytes"),
        }
    }
}

impl std::error::Error for Error {}

/// Validates a strftime format string for correct syntax.
/// This performs a basic validation to catch common errors.
///
/// Returns Ok(()) if the format appears valid, or an error describing the issue.
pub fn validate_format(format: impl AsRef<str>) -> Result<(), Error> {
    let format = format.as_ref();

    // Check for empty format
    if format.is_empty() {
        return Err(Error::InvalidFormatString);
    }

    // Check for null bytes (which would cause CString creation to fail)
    if format.contains('\0') {
        return Err(Error::NullByteError);
    }

    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        // Look for % sequences
        if c == '%' {
            match chars.next() {
                // These are the most common format specifiers
                Some('a') | Some('A') | Some('b') | Some('B') | Some('c') | Some('C')
                | Some('d') | Some('D') | Some('e') | Some('F') | Some('g') | Some('G')
                | Some('h') | Some('H') | Some('I') | Some('j') | Some('k') | Some('l')
                | Some('m') | Some('M') | Some('n') | Some('p') | Some('P') | Some('r')
                | Some('R') | Some('s') | Some('S') | Some('t') | Some('T') | Some('u')
                | Some('U') | Some('V') | Some('w') | Some('W') | Some('x') | Some('X')
                | Some('y') | Some('Y') | Some('z') | Some('Z') | Some('%') | Some('E')
                | Some('O') | Some('+') => {
                    // Valid format specifier
                    continue;
                }
                Some(_c) => {
                    // Unknown format specifier
                    return Err(Error::InvalidFormatString);
                }
                None => {
                    // % at end of string
                    return Err(Error::InvalidFormatString);
                }
            }
        }
    }

    // Check for the special {ms} sequence format
    let ms_braces = format.match_indices('{').count();
    let ms_closing_braces = format.match_indices('}').count();
    if ms_braces != ms_closing_braces {
        return Err(Error::InvalidFormatString);
    }

    Ok(())
}

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

/// Split a timestamp into its components in UTC timezone.
pub fn components_utc(ts_seconds: TimeStamp) -> Result<Components, Error> {
    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { gmtime_r(&ts_seconds, tm.as_mut_ptr()) }.is_null() {
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

/// Split a timestamp into its components in the local timezone.
pub fn components_local(ts_seconds: TimeStamp) -> Result<Components, Error> {
    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { localtime_r(&ts_seconds, tm.as_mut_ptr()) }.is_null() {
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

/// Convert a `std::time::SystemTime` to a UNIX timestamp in seconds.
///
/// This function converts a `std::time::SystemTime` instance to a `TimeStamp` (Unix timestamp in seconds).
/// It handles the conversion and error cases related to negative timestamps or other time conversion issues.
///
/// # Examples
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH, Duration};
///
/// // Convert the current system time to a timestamp
/// let system_time = SystemTime::now();
/// let timestamp = time_format::from_system_time(system_time).unwrap();
///
/// // Convert a specific time
/// let past_time = UNIX_EPOCH + Duration::from_secs(1500000000);
/// let past_timestamp = time_format::from_system_time(past_time).unwrap();
/// assert_eq!(past_timestamp, 1500000000);
/// ```
///
/// ## Working with Time Components
///
/// You can use the function to convert a `SystemTime` to components:
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH, Duration};
///
/// // Create a specific time: January 15, 2023 at 14:30:45 UTC
/// let specific_time = UNIX_EPOCH + Duration::from_secs(1673793045);
///
/// // Convert to timestamp
/// let ts = time_format::from_system_time(specific_time).unwrap();
///
/// // Get the time components
/// let components = time_format::components_utc(ts).unwrap();
///
/// // Verify the time components
/// assert_eq!(components.year, 2023);
/// assert_eq!(components.month, 1); // January
/// assert_eq!(components.month_day, 15);
/// assert_eq!(components.hour, 14);
/// assert_eq!(components.min, 30);
/// assert_eq!(components.sec, 45);
/// ```
///
/// ## Formatting with strftime
///
/// Convert a `SystemTime` and format it as a string:
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH, Duration};
///
/// // Create a specific time
/// let specific_time = UNIX_EPOCH + Duration::from_secs(1673793045);
///
/// // Convert to timestamp
/// let ts = time_format::from_system_time(specific_time).unwrap();
///
/// // Format as ISO 8601
/// let iso8601 = time_format::format_iso8601_utc(ts).unwrap();
/// assert_eq!(iso8601, "2023-01-15T14:30:45Z");
///
/// // Custom formatting
/// let custom_format = time_format::strftime_utc("%B %d, %Y at %H:%M:%S", ts).unwrap();
/// assert_eq!(custom_format, "January 15, 2023 at 14:30:45");
/// ```
pub fn from_system_time(time: std::time::SystemTime) -> Result<TimeStamp, Error> {
    time.duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::TimeError)?
        .as_secs()
        .try_into()
        .map_err(|_| Error::InvalidTimestamp)
}

/// Return the current UNIX timestamp in seconds.
pub fn now() -> Result<TimeStamp, Error> {
    from_system_time(std::time::SystemTime::now())
}

/// Convert a `std::time::SystemTime` to a UNIX timestamp with millisecond precision.
///
/// This function converts a `std::time::SystemTime` instance to a `TimeStampMs` (Unix timestamp with millisecond precision).
/// It extracts both the seconds and milliseconds components from the system time.
///
/// # Examples
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH, Duration};
///
/// // Convert the current system time to a timestamp with millisecond precision
/// let system_time = SystemTime::now();
/// let timestamp_ms = time_format::from_system_time_ms(system_time).unwrap();
/// println!("Seconds: {}, Milliseconds: {}", timestamp_ms.seconds, timestamp_ms.milliseconds);
///
/// // Convert a specific time with millisecond precision
/// let specific_time = UNIX_EPOCH + Duration::from_millis(1500000123);
/// let specific_ts_ms = time_format::from_system_time_ms(specific_time).unwrap();
/// assert_eq!(specific_ts_ms.seconds, 1500000);
/// assert_eq!(specific_ts_ms.milliseconds, 123);
/// ```
///
/// ## Using with TimeStampMs methods
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH, Duration};
///
/// // Create a precise time: 1500000 seconds and 123 milliseconds after the epoch
/// let specific_time = UNIX_EPOCH + Duration::from_millis(1500000123);
///
/// // Convert to TimeStampMs
/// let ts_ms = time_format::from_system_time_ms(specific_time).unwrap();
///
/// // Get total milliseconds
/// let total_ms = ts_ms.total_milliseconds();
/// assert_eq!(total_ms, 1500000123);
/// ```
///
/// ## Formatting timestamps with millisecond precision
///
/// You can format a timestamp with millisecond precision:
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH, Duration};
///
/// // Create a specific timestamp with millisecond precision
/// // We'll use a fixed timestamp rather than a date calculation to avoid test failures
/// let ts_ms = time_format::TimeStampMs::new(1743087045, 678);
///
/// // Format with milliseconds using your preferred pattern
/// let formatted = time_format::strftime_ms_utc("%Y-%m-%d %H:%M:%S.{ms}", ts_ms).unwrap();
///
/// // Verify the milliseconds are included
/// assert!(formatted.contains(".678"));
///
/// // Format as ISO 8601 with milliseconds
/// let iso8601_ms = time_format::format_iso8601_ms_utc(ts_ms).unwrap();
/// assert!(iso8601_ms.ends_with(".678Z"));
///
/// // Use with common date formats
/// let rfc3339 = time_format::format_common_ms_utc(ts_ms, time_format::DateFormat::RFC3339).unwrap();
/// assert!(rfc3339.contains(".678"));
/// ```
///
/// ## Converting between TimeStamp and TimeStampMs
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH, Duration};
///
/// // Create a system time with millisecond precision
/// let system_time = UNIX_EPOCH + Duration::from_millis(1673793045678);
///
/// // Convert to TimeStampMs
/// let ts_ms = time_format::from_system_time_ms(system_time).unwrap();
/// assert_eq!(ts_ms.seconds, 1673793045);
/// assert_eq!(ts_ms.milliseconds, 678);
///
/// // Convert to TimeStamp (loses millisecond precision)
/// let ts = time_format::from_system_time(system_time).unwrap();
/// assert_eq!(ts, 1673793045);
///
/// // Convert from TimeStamp to TimeStampMs
/// let ts_ms_from_ts = time_format::TimeStampMs::from_timestamp(ts);
/// assert_eq!(ts_ms_from_ts.seconds, ts);
/// assert_eq!(ts_ms_from_ts.milliseconds, 0); // milliseconds are lost
/// ```
pub fn from_system_time_ms(time: std::time::SystemTime) -> Result<TimeStampMs, Error> {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::TimeError)?;

    let seconds = duration
        .as_secs()
        .try_into()
        .map_err(|_| Error::InvalidTimestamp)?;
    let millis = duration.subsec_millis() as u16;

    Ok(TimeStampMs::new(seconds, millis))
}

/// Return the current UNIX timestamp with millisecond precision.
pub fn now_ms() -> Result<TimeStampMs, Error> {
    from_system_time_ms(std::time::SystemTime::now())
}

/// Return the current time in the specified format, in the UTC time zone.
/// The time is assumed to be the number of seconds since the Epoch.
///
/// This function will validate the format string before attempting to format the time.
pub fn strftime_utc(format: impl AsRef<str>, ts_seconds: TimeStamp) -> Result<String, Error> {
    let format = format.as_ref();

    // Validate the format string
    validate_format(format)?;

    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { gmtime_r(&ts_seconds, tm.as_mut_ptr()) }.is_null() {
        return Err(Error::TimeError);
    }
    let tm = unsafe { tm.assume_init() };

    format_time_with_tm(format, &tm)
}

/// Return the current time in the specified format, in the local time zone.
/// The time is assumed to be the number of seconds since the Epoch.
///
/// This function will validate the format string before attempting to format the time.
pub fn strftime_local(format: impl AsRef<str>, ts_seconds: TimeStamp) -> Result<String, Error> {
    let format = format.as_ref();

    // Validate the format string
    validate_format(format)?;

    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { localtime_r(&ts_seconds, tm.as_mut_ptr()) }.is_null() {
        return Err(Error::TimeError);
    }
    let tm = unsafe { tm.assume_init() };

    format_time_with_tm(format, &tm)
}

// Internal helper function to format time with a tm struct
fn format_time_with_tm(format: &str, tm: &tm) -> Result<String, Error> {
    let format_len = format.len();
    let format = CString::new(format).map_err(|_| Error::NullByteError)?;
    let mut buf_size = format_len;
    let mut buf: Vec<u8> = vec![0; buf_size];

    // Initial attempt
    let mut len = unsafe {
        strftime(
            buf.as_mut_ptr() as *mut c_char,
            buf_size,
            format.as_ptr() as *const c_char,
            tm,
        )
    };

    // If the format is invalid, strftime returns 0 but won't use more buffer space
    // We try once with a much larger buffer to distinguish between these cases
    if len == 0 {
        // Try with a larger buffer first
        buf_size *= 10;
        buf.resize(buf_size, 0);

        len = unsafe {
            strftime(
                buf.as_mut_ptr() as *mut c_char,
                buf_size,
                format.as_ptr() as *const c_char,
                tm,
            )
        };

        // If still 0 with a much larger buffer, it's likely an invalid format
        if len == 0 {
            return Err(Error::InvalidFormatString);
        }
    }

    // Keep growing the buffer if needed
    while len == 0 {
        buf_size *= 2;
        buf.resize(buf_size, 0);
        len = unsafe {
            strftime(
                buf.as_mut_ptr() as *mut c_char,
                buf_size,
                format.as_ptr() as *const c_char,
                tm,
            )
        };
    }

    buf.truncate(len);
    String::from_utf8(buf).map_err(|_| Error::Utf8Error)
}

/// Return the current time in the specified format, in the UTC time zone,
/// with support for custom millisecond formatting.
///
/// The standard format directives from strftime are supported.
/// Additionally, the special text sequence '{ms}' will be replaced with the millisecond component.
///
/// Example: strftime_ms_utc("%Y-%m-%d %H:%M:%S.{ms}", ts_ms)
///
/// This function will validate the format string before attempting to format the time.
pub fn strftime_ms_utc(format: impl AsRef<str>, ts_ms: TimeStampMs) -> Result<String, Error> {
    let format_str = format.as_ref();

    // Validate the format string (validation also checks for balanced braces)
    validate_format(format_str)?;

    // First, format the seconds part
    // Skip validation in strftime_utc since we already did it
    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { gmtime_r(&ts_ms.seconds, tm.as_mut_ptr()) }.is_null() {
        return Err(Error::TimeError);
    }
    let tm = unsafe { tm.assume_init() };

    let seconds_formatted = format_time_with_tm(format_str, &tm)?;

    // If the format contains the {ms} placeholder, replace it with the milliseconds
    if format_str.contains("{ms}") {
        // Format milliseconds with leading zeros
        let ms_str = format!("{:03}", ts_ms.milliseconds);
        Ok(seconds_formatted.replace("{ms}", &ms_str))
    } else {
        Ok(seconds_formatted)
    }
}

/// Return the current time in the specified format, in the local time zone,
/// with support for custom millisecond formatting.
///
/// The standard format directives from strftime are supported.
/// Additionally, the special text sequence '{ms}' will be replaced with the millisecond component.
///
/// Example: strftime_ms_local("%Y-%m-%d %H:%M:%S.{ms}", ts_ms)
///
/// This function will validate the format string before attempting to format the time.
pub fn strftime_ms_local(format: impl AsRef<str>, ts_ms: TimeStampMs) -> Result<String, Error> {
    let format_str = format.as_ref();

    // Validate the format string (validation also checks for balanced braces)
    validate_format(format_str)?;

    // First, format the seconds part
    // Skip validation in strftime_local since we already did it
    let mut tm = MaybeUninit::<tm>::uninit();
    if unsafe { localtime_r(&ts_ms.seconds, tm.as_mut_ptr()) }.is_null() {
        return Err(Error::TimeError);
    }
    let tm = unsafe { tm.assume_init() };

    let seconds_formatted = format_time_with_tm(format_str, &tm)?;

    // If the format contains the {ms} placeholder, replace it with the milliseconds
    if format_str.contains("{ms}") {
        // Format milliseconds with leading zeros
        let ms_str = format!("{:03}", ts_ms.milliseconds);
        Ok(seconds_formatted.replace("{ms}", &ms_str))
    } else {
        Ok(seconds_formatted)
    }
}

/// Format a timestamp according to ISO 8601 format in UTC.
///
/// ISO 8601 is an international standard for date and time representations.
/// This function returns the timestamp in the format: `YYYY-MM-DDThh:mm:ssZ`
///
/// Example: "2025-05-20T14:30:45Z"
///
/// For more details on ISO 8601, see: https://en.wikipedia.org/wiki/ISO_8601
pub fn format_iso8601_utc(ts: TimeStamp) -> Result<String, Error> {
    strftime_utc("%Y-%m-%dT%H:%M:%SZ", ts)
}

/// Format a timestamp with millisecond precision according to ISO 8601 format in UTC.
///
/// ISO 8601 is an international standard for date and time representations.
/// This function returns the timestamp in the format: `YYYY-MM-DDThh:mm:ss.sssZ`
///
/// Example: "2025-05-20T14:30:45.123Z"
///
/// For more details on ISO 8601, see: https://en.wikipedia.org/wiki/ISO_8601
pub fn format_iso8601_ms_utc(ts_ms: TimeStampMs) -> Result<String, Error> {
    strftime_ms_utc("%Y-%m-%dT%H:%M:%S.{ms}Z", ts_ms)
}

/// Format a timestamp according to ISO 8601 format in the local timezone.
///
/// This function returns the timestamp in the format: `YYYY-MM-DDThh:mm:ss±hh:mm`
/// where the `±hh:mm` part represents the timezone offset from UTC.
///
/// Example: "2025-05-20T09:30:45-05:00"
///
/// For more details on ISO 8601, see: https://en.wikipedia.org/wiki/ISO_8601
pub fn format_iso8601_local(ts: TimeStamp) -> Result<String, Error> {
    strftime_local("%Y-%m-%dT%H:%M:%S%z", ts).map(|s| {
        // Standard ISO 8601 requires a colon in timezone offset (e.g., -05:00 not -0500)
        // But strftime just gives us -0500, so we need to insert the colon
        if s.len() > 5 && (s.ends_with('0') || s.chars().last().unwrap().is_ascii_digit()) {
            let len = s.len();
            format!("{}:{}", &s[..len - 2], &s[len - 2..])
        } else {
            s
        }
    })
}

/// Format a timestamp with millisecond precision according to ISO 8601 format in the local timezone.
///
/// This function returns the timestamp in the format: `YYYY-MM-DDThh:mm:ss.sss±hh:mm`
/// where the `±hh:mm` part represents the timezone offset from UTC.
///
/// Example: "2025-05-20T09:30:45.123-05:00"
///
/// For more details on ISO 8601, see: https://en.wikipedia.org/wiki/ISO_8601
pub fn format_iso8601_ms_local(ts_ms: TimeStampMs) -> Result<String, Error> {
    strftime_ms_local("%Y-%m-%dT%H:%M:%S.{ms}%z", ts_ms).map(|s| {
        // Insert colon in timezone offset for ISO 8601 compliance
        let len = s.len();
        if len > 5 && (s.ends_with('0') || s.chars().last().unwrap().is_ascii_digit()) {
            format!("{}:{}", &s[..len - 2], &s[len - 2..])
        } else {
            s
        }
    })
}

/// Format types for common date strings
///
/// This enum provides common date and time format patterns.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DateFormat {
    /// RFC 3339 (similar to ISO 8601) format: "2025-05-20T14:30:45Z" or "2025-05-20T14:30:45-05:00"
    RFC3339,
    /// RFC 2822 format: "Tue, 20 May 2025 14:30:45 -0500"
    RFC2822,
    /// HTTP format (RFC 7231): "Tue, 20 May 2025 14:30:45 GMT"
    HTTP,
    /// SQL format: "2025-05-20 14:30:45"
    SQL,
    /// US date format: "05/20/2025 02:30:45 PM"
    US,
    /// European date format: "20/05/2025 14:30:45"
    European,
    /// Short date: "05/20/25"
    ShortDate,
    /// Long date: "Tuesday, May 20, 2025"
    LongDate,
    /// Short time: "14:30"
    ShortTime,
    /// Long time: "14:30:45"
    LongTime,
    /// Date and time: "2025-05-20 14:30:45"
    DateTime,
    /// Custom format string
    Custom(&'static str),
}

impl DateFormat {
    /// Get the format string for this format
    fn get_format_string(&self) -> &'static str {
        match self {
            Self::RFC3339 => "%Y-%m-%dT%H:%M:%S%z",
            Self::RFC2822 => "%a, %d %b %Y %H:%M:%S %z",
            Self::HTTP => "%a, %d %b %Y %H:%M:%S GMT",
            Self::SQL => "%Y-%m-%d %H:%M:%S",
            Self::US => "%m/%d/%Y %I:%M:%S %p",
            Self::European => "%d/%m/%Y %H:%M:%S",
            Self::ShortDate => "%m/%d/%y",
            Self::LongDate => "%A, %B %d, %Y",
            Self::ShortTime => "%H:%M",
            Self::LongTime => "%H:%M:%S",
            Self::DateTime => "%Y-%m-%d %H:%M:%S",
            Self::Custom(fmt) => fmt,
        }
    }
}

/// Format a timestamp using a common date format in UTC timezone
///
/// Examples:
/// ```rust
/// let ts = time_format::now().unwrap();
///
/// // Format as RFC 3339
/// let rfc3339 = time_format::format_common_utc(ts, time_format::DateFormat::RFC3339).unwrap();
///
/// // Format as HTTP date
/// let http_date = time_format::format_common_utc(ts, time_format::DateFormat::HTTP).unwrap();
///
/// // Format with a custom format
/// let custom = time_format::format_common_utc(ts, time_format::DateFormat::Custom("%Y-%m-%d")).unwrap();
/// ```
pub fn format_common_utc(ts: TimeStamp, format: DateFormat) -> Result<String, Error> {
    let format_str = format.get_format_string();

    match format {
        DateFormat::RFC3339 => {
            // Handle RFC3339 specially to ensure proper timezone formatting
            strftime_utc(format_str, ts).map(|s| {
                if s.ends_with('0') || s.chars().last().unwrap().is_ascii_digit() {
                    let len = s.len();
                    format!("{}:{}", &s[..len - 2], &s[len - 2..])
                } else {
                    s
                }
            })
        }
        _ => strftime_utc(format_str, ts),
    }
}

/// Format a timestamp using a common date format in local timezone
///
/// Examples:
/// ```rust
/// let ts = time_format::now().unwrap();
///
/// // Format as RFC 2822
/// let rfc2822 = time_format::format_common_local(ts, time_format::DateFormat::RFC2822).unwrap();
///
/// // Format as US date
/// let us_date = time_format::format_common_local(ts, time_format::DateFormat::US).unwrap();
/// ```
pub fn format_common_local(ts: TimeStamp, format: DateFormat) -> Result<String, Error> {
    let format_str = format.get_format_string();

    match format {
        DateFormat::RFC3339 => {
            // Handle RFC3339 specially to ensure proper timezone formatting
            strftime_local(format_str, ts).map(|s| {
                if s.ends_with('0') || s.chars().last().unwrap().is_ascii_digit() {
                    let len = s.len();
                    format!("{}:{}", &s[..len - 2], &s[len - 2..])
                } else {
                    s
                }
            })
        }
        DateFormat::HTTP => {
            // HTTP dates are always in GMT/UTC, so redirect to the UTC version
            format_common_utc(ts, format)
        }
        _ => strftime_local(format_str, ts),
    }
}

/// Format a timestamp with millisecond precision using a common date format in UTC timezone
///
/// This function extends common date formats to include milliseconds where appropriate.
/// For formats that don't typically include milliseconds (like ShortDate), the milliseconds are ignored.
///
/// Examples:
/// ```rust
/// let ts_ms = time_format::now_ms().unwrap();
///
/// // Format as RFC 3339 with milliseconds
/// let rfc3339 = time_format::format_common_ms_utc(ts_ms, time_format::DateFormat::RFC3339).unwrap();
/// // Example: "2025-05-20T14:30:45.123Z"
/// ```
pub fn format_common_ms_utc(ts_ms: TimeStampMs, format: DateFormat) -> Result<String, Error> {
    // For formats that can reasonably include milliseconds, add them
    let format_str = match format {
        DateFormat::RFC3339 => "%Y-%m-%dT%H:%M:%S.{ms}%z",
        DateFormat::SQL => "%Y-%m-%d %H:%M:%S.{ms}",
        DateFormat::DateTime => "%Y-%m-%d %H:%M:%S.{ms}",
        DateFormat::LongTime => "%H:%M:%S.{ms}",
        DateFormat::Custom(fmt) => fmt,
        _ => format.get_format_string(), // Use standard format for others
    };

    match format {
        DateFormat::RFC3339 => {
            // Handle RFC3339 specially for timezone formatting
            strftime_ms_utc(format_str, ts_ms).map(|s| {
                if s.ends_with('0') || s.chars().last().unwrap().is_ascii_digit() {
                    let len = s.len();
                    format!("{}:{}", &s[..len - 2], &s[len - 2..])
                } else {
                    s
                }
            })
        }
        _ => strftime_ms_utc(format_str, ts_ms),
    }
}

/// Format a timestamp with millisecond precision using a common date format in local timezone
///
/// This function extends common date formats to include milliseconds where appropriate.
/// For formats that don't typically include milliseconds (like ShortDate), the milliseconds are ignored.
///
/// Examples:
/// ```rust
/// let ts_ms = time_format::now_ms().unwrap();
///
/// // Format as RFC 3339 with milliseconds in local time
/// let local_time = time_format::format_common_ms_local(ts_ms, time_format::DateFormat::RFC3339).unwrap();
/// // Example: "2025-05-20T09:30:45.123-05:00"
/// ```
pub fn format_common_ms_local(ts_ms: TimeStampMs, format: DateFormat) -> Result<String, Error> {
    // For formats that can reasonably include milliseconds, add them
    let format_str = match format {
        DateFormat::RFC3339 => "%Y-%m-%dT%H:%M:%S.{ms}%z",
        DateFormat::SQL => "%Y-%m-%d %H:%M:%S.{ms}",
        DateFormat::DateTime => "%Y-%m-%d %H:%M:%S.{ms}",
        DateFormat::LongTime => "%H:%M:%S.{ms}",
        DateFormat::Custom(fmt) => fmt,
        _ => format.get_format_string(), // Use standard format for others
    };

    match format {
        DateFormat::RFC3339 => {
            // Handle RFC3339 specially for timezone formatting
            strftime_ms_local(format_str, ts_ms).map(|s| {
                if s.ends_with('0') || s.chars().last().unwrap().is_ascii_digit() {
                    let len = s.len();
                    format!("{}:{}", &s[..len - 2], &s[len - 2..])
                } else {
                    s
                }
            })
        }
        DateFormat::HTTP => {
            // HTTP dates are always in GMT/UTC, so redirect to the UTC version
            format_common_ms_utc(ts_ms, format)
        }
        _ => strftime_ms_local(format_str, ts_ms),
    }
}
