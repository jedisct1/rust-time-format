# time-format

This crate does only one thing: format a Unix timestamp.

## Splitting a timestamp into its components

The `components_utc()` function returns the components of a timestamp:

```rust
let ts = time_format::now().unwrap();
let components = time_format::components_utc(ts).unwrap();
```

Components are `sec`, `min`, `hour`, `month_day`, `month`, `year`, `week_day` and `year_day`.

## Formatting a timestamp

The `strftime_utc()` function formats a timestamp, using the same format as the `strftime()` function of the standard C library.

```rust
let ts = time_format::now().unwrap();
let s = time_format::strftime_utc("%Y-%m-%d", ts).unwrap();
```

## That's it

If you need a minimal crate to get timestamps and perform basic operations on them, check out [coarsetime](https://crates.io/crates/coarsetime).

`coarsetime` fully supports WebAssembly, in browsers and WASI environments.
