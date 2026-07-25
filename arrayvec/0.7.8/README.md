
arrayvec
========
[![Crates.io: arrayvec](https://img.shields.io/crates/v/arrayvec.svg)](https://crates.io/crates/arrayvec)
[![Documentation](https://docs.rs/arrayvec/badge.svg)](https://docs.rs/arrayvec)
[![Build Status](https://github.com/bluss/arrayvec/workflows/Continuous%20integration/badge.svg?branch=master)](https://github.com/bluss/arrayvec/actions)




[![License: Apache](https://img.shields.io/badge/License-Apache%202.0-red.svg)](LICENSE-APACHE)
OR
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A vector with fixed capacity.

Please read the [`API documentation here`](https://docs.rs/arrayvec)

## Creusot verification status

The common initialized-prefix pilot proves the bodies of `push`, `try_push`,
`push_unchecked`, and `pop` against ordered-element and ownership contracts.
The consuming iterator has a strong sequence protocol, but its moved-hole
representation bridge is still explicitly trusted. See
[`../../UNSAFE_COLLECTION_VERIFICATION.md`](../../UNSAFE_COLLECTION_VERIFICATION.md)
for the status table, trusted removal conditions, and scoped commands.

# License

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
