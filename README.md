# time-format

[![Crates.io](https://img.shields.io/crates/v/time-format.svg)](https://crates.io/crates/time-format)
[![Documentation](https://docs.rs/time-format/badge.svg)](https://docs.rs/time-format)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A lightweight, zero-dependency Rust crate for formatting Unix timestamps in both UTC and local time, with millisecond precision.

## Features

- 🚀 **Zero dependencies** - No external crates needed
- 🌐 **Multiple timezones** - Support for both UTC and local time zones
- ⏱️ **Millisecond precision** - High-resolution timestamp formatting 
- 🧩 **Component access** - Split timestamps into their individual components
- 📅 **Standard formats** - Built-in support for ISO 8601, RFC 3339, RFC 2822, and more
- 🔄 **Custom formatting** - Flexible formatting using C's `strftime` patterns
- ⚡ **Performance** - Direct FFI bindings to system time functions

## Why time-format?

**Simple things should be simple.**

When you just need to format a timestamp in a standardized way like ISO 8601 or get time components, you shouldn't need to pull in complex dependencies, understand type conversion hierarchies, or deal with feature flags.

**time-format** excels at:

- **Minimalism**: Zero dependencies means faster builds and smaller binaries
- **Simplicity**: Clear and intuitive API with straightforward error handling
- **Performance**: Thin wrapper over system time functions for minimal overhead
- **Common formats**: Built-in support for the most widely used date formats

It's the ideal choice when you just need timestamp formatting without the complexity of full-featured time libraries.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
time-format = "1.2.0"
```

## Basic Usage

### Getting the Current Time

```rust
// Get current time in seconds
let ts = time_format::now().unwrap();

// Get current time with millisecond precision
let ts_ms = time_format::now_ms().unwrap();
```

### Splitting a Timestamp into Components

```rust
// Get components in UTC
let ts = time_format::now().unwrap();
let components = time_format::components_utc(ts).unwrap();
println!("Current hour: {}", components.hour);

// Available components:
// - sec (0-59)
// - min (0-59)
// - hour (0-23)
// - month_day (1-31)
// - month (1-12) - January is 1, December is 12
// - year (e.g., 2025)
// - week_day (0-6) - Sunday is 0, Saturday is 6
// - year_day (0-365)

// Get components in local time
let local_components = time_format::components_local(ts).unwrap();
```

### Formatting a Timestamp

#### UTC Time

```rust
let ts = time_format::now().unwrap();

// Basic date formatting
let date = time_format::strftime_utc("%Y-%m-%d", ts).unwrap();
// Example: "2025-05-20"

// Time formatting
let time = time_format::strftime_utc("%H:%M:%S", ts).unwrap();
// Example: "14:30:45"

// Combined date and time
let datetime = time_format::strftime_utc("%Y-%m-%d %H:%M:%S", ts).unwrap();
// Example: "2025-05-20 14:30:45"

// Custom format
let custom = time_format::strftime_utc("%a, %d %b %Y %T %Z", ts).unwrap();
// Example: "Tue, 20 May 2025 14:30:45 UTC"
```

#### Local Time

```rust
let ts = time_format::now().unwrap();

// Local time with timezone name
let local_time = time_format::strftime_local("%Y-%m-%d %H:%M:%S %Z", ts).unwrap();
// Example: "2025-05-20 09:30:45 PDT"
```

#### Millisecond Precision

```rust
let ts_ms = time_format::now_ms().unwrap();

// Format with milliseconds in UTC
let precise_time = time_format::strftime_ms_utc("%Y-%m-%d %H:%M:%S.{ms}", ts_ms).unwrap();
// Example: "2025-05-20 14:30:45.123"

// Format with milliseconds in local time
let precise_local = time_format::strftime_ms_local("%Y-%m-%d %H:%M:%S.{ms} %Z", ts_ms).unwrap();
// Example: "2025-05-20 09:30:45.123 PDT"
```

### ISO 8601 Formatting

Format timestamps according to ISO 8601 standard:

```rust
let ts = time_format::now().unwrap();
let ts_ms = time_format::now_ms().unwrap();

// ISO 8601 in UTC
let iso8601 = time_format::format_iso8601_utc(ts).unwrap();
// Example: "2025-05-20T14:30:45Z"

// ISO 8601 with milliseconds in UTC
let iso8601_ms = time_format::format_iso8601_ms_utc(ts_ms).unwrap();
// Example: "2025-05-20T14:30:45.123Z"

// ISO 8601 in local timezone
let iso8601_local = time_format::format_iso8601_local(ts).unwrap();
// Example: "2025-05-20T09:30:45-05:00"

// ISO 8601 with milliseconds in local timezone
let iso8601_ms_local = time_format::format_iso8601_ms_local(ts_ms).unwrap();
// Example: "2025-05-20T09:30:45.123-05:00"
```

### Common Date Formats

The crate provides convenient formatting for common date formats:

```rust
let ts = time_format::now().unwrap();
let ts_ms = time_format::now_ms().unwrap();

// RFC 3339 (similar to ISO 8601)
let rfc3339 = time_format::format_common_utc(ts, time_format::DateFormat::RFC3339).unwrap();
// Example: "2025-05-20T14:30:45Z"

// RFC 2822 format (email format)
let rfc2822 = time_format::format_common_utc(ts, time_format::DateFormat::RFC2822).unwrap();
// Example: "Tue, 20 May 2025 14:30:45 +0000"

// HTTP date format (RFC 7231)
let http_date = time_format::format_common_utc(ts, time_format::DateFormat::HTTP).unwrap();
// Example: "Tue, 20 May 2025 14:30:45 GMT"

// SQL date format
let sql_format = time_format::format_common_utc(ts, time_format::DateFormat::SQL).unwrap();
// Example: "2025-05-20 14:30:45"

// US date format
let us_format = time_format::format_common_local(ts, time_format::DateFormat::US).unwrap();
// Example: "05/20/2025 02:30:45 PM"

// European date format
let eu_format = time_format::format_common_local(ts, time_format::DateFormat::European).unwrap();
// Example: "20/05/2025 14:30:45"

// With millisecond precision
let rfc3339_ms = time_format::format_common_ms_utc(ts_ms, time_format::DateFormat::RFC3339).unwrap();
// Example: "2025-05-20T14:30:45.123Z"

// Custom format
let custom = time_format::format_common_utc(ts, time_format::DateFormat::Custom("%Y/%m/%d")).unwrap();
// Example: "2025/05/20"
```

Available format types:
- `RFC3339`: ISO 8601-like format
- `RFC2822`: Email date format
- `HTTP`: Web standard date format
- `SQL`: SQL database format
- `US`: US style date format (MM/DD/YYYY)
- `European`: European style date format (DD/MM/YYYY)
- `ShortDate`: Short date (MM/DD/YY)
- `LongDate`: Long date with full month and day names
- `ShortTime`: Hours and minutes
- `LongTime`: Hours, minutes, and seconds
- `DateTime`: ISO-like date and time
- `Custom`: Custom format string

## Common Format Directives

| Directive | Description                     | Example                  |
| --------- | ------------------------------- | ------------------------ |
| `%Y`      | Year (4 digits)                 | 2025                     |
| `%m`      | Month (01-12)                   | 05                       |
| `%d`      | Day of month (01-31)            | 20                       |
| `%H`      | Hour (00-23)                    | 14                       |
| `%M`      | Minute (00-59)                  | 30                       |
| `%S`      | Second (00-59)                  | 45                       |
| `%a`      | Abbreviated weekday             | Tue                      |
| `%A`      | Full weekday                    | Tuesday                  |
| `%b`      | Abbreviated month               | May                      |
| `%B`      | Full month                      | May                      |
| `%c`      | Locale date and time            | Tue May 20 14:30:45 2025 |
| `%Z`      | Timezone name                   | UTC, PDT, etc.           |
| `%z`      | Timezone offset                 | +0000, -0500             |
| `{ms}`    | Milliseconds (custom extension) | 123                      |

## Comparison with Other Time Libraries

| Feature            | time-format | chrono   | time     |
| ------------------ | ----------- | -------- | -------- |
| Dependencies       | None        | Multiple | Multiple |
| ISO 8601           | ✅           | ✅        | ✅        |
| RFC 2822/3339      | ✅           | ✅        | ✅        |
| Milliseconds       | ✅           | ✅        | ✅        |
| Local timezone     | ✅           | ✅        | ✅        |
| Custom formats     | ✅           | ✅        | ✅        |
| Binary size impact | Very Small  | Larger   | Medium   |
| Compile time       | Fast        | Slower   | Medium   |

`time-format` is designed to be a lightweight alternative when you only need basic timestamp formatting capabilities without pulling in additional dependencies.

## Related Crates

- [coarsetime](https://crates.io/crates/coarsetime) - A minimal crate to get timestamps and perform basic operations on them
- [chrono](https://crates.io/crates/chrono) - Complete date and time library with calendar feature support
- [time](https://crates.io/crates/time) - A comprehensive time library with timezone database support

## Examples

### Working with Web APIs

```rust
use time_format::{format_iso8601_utc, now};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get current time
    let current_time = now()?;
    
    // Format as ISO 8601 for API request
    let timestamp = format_iso8601_utc(current_time)?;
    
    println!("API Request time: {}", timestamp);
    // Example output: "2025-05-20T14:30:45Z"
    
    // Now you can use this timestamp in your API requests
    // let response = make_api_request(timestamp);
    
    Ok(())
}
```

### Logging with Timestamps

```rust
use time_format::{DateFormat, format_common_ms_local, now_ms};

fn log_message(level: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Get current time with millisecond precision
    let current_time = now_ms()?;
    
    // Format with milliseconds for logging
    let timestamp = format_common_ms_local(current_time, DateFormat::DateTime)?;
    
    println!("[{}] {} - {}", timestamp, level, message);
    // Example output: "[2025-05-20 09:30:45.123] INFO - Application started"
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_message("INFO", "Application started")?;
    log_message("DEBUG", "Configuration loaded")?;
    
    Ok(())
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [MIT License](LICENSE).

---

*Note: This crate uses FFI bindings to C's time functions. It's designed to be lightweight and efficient, but it does not include a timezone database. For applications requiring extensive timezone handling, consider `chrono` or `time`.*
