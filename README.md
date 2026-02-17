# noterm

[![crates.io](https://img.shields.io/crates/d/noterm.svg)](https://crates.io/crates/noterm)
[![crates.io](https://img.shields.io/crates/v/noterm.svg)](https://crates.io/crates/noterm)
[![Documentation](https://docs.rs/noterm/badge.svg)](https://docs.rs/noterm)

`noterm`, a `no-std` terminal handler.

# Rationale

This crate provides a working but yet minimal implementation of a terminal handler. It could be used
for rendering data and controling events in a VT100/Kitty compatible terminal.

This crate does not rely on crate `alloc` but `heapless` for the result of argument parsing. This
could be mitigated and improved in a near future. As such, it could be used in very constrained and
critical embedded developments.

# Example

To be redacted.

# Status

This crate is still a work in progress and is subject to huge changes in its API.

# Roadmap

To be redacted.

Please feel free to email me with suggestions or directly propose a Pull Request with some valuable
contribution. As it is the beginning of the project, I will take time to studi every contribution.

# License

This work is licensed under either

- APACHE License, version 2.0
- MIT License

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

# Credits

I would like to give a big thank to the creator and contributors of the crate
[`crossterm`](https://github.com/crossterm-rs/crossterm), which I draw a lot of inspiration from.
