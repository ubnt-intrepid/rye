<h1 align="center">
  <code>rye</code>
</h1>
<div align="center">
  <strong>
    A custom unit testing framework inspired by Catch2.
  </strong>
</div>

<br />

<div align="center">
  <a href="https://crates.io/crates/rye">
    <img src="https://img.shields.io/crates/v/rye.svg?style=flat-square"
         alt="crates.io"
    />
  </a>
  <a href="https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html">
    <img src="https://img.shields.io/badge/rust-1.40.0-gray?style=flat-square"
         alt="rust toolchain"
    />
  </a>
  <a href="https://docs.rs/rye">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
         alt="docs.rs" />
  </a>
</div>

<br />

The goal of this project is to provide a unit testing framework for Rust that focuses on the simplicity and
reusability of test codes.
The concept is heavily influenced by the section mechanism in [`Catch2`](https://github.com/catchorg/Catch2),
a C++ unit testing framework library.

> **WARNING:** This library is currently on the experimental stage and cannot be used for production use.
> Some major changes may occur until releasing 0.1.0.

## Installation

Add `rye` to `dev-dependencies` section in your `Cargo.toml` as follows:

```toml
[dev-dependencies]
rye = "0.0.1"
```

## Resources

* [API documentation (docs.rs)](https://docs.rs/rye)
* [API documentation (master)](https://ubnt-intrepid.github.io/rye/rye/index.html)

## License

This library is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
