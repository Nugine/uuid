// Copyright 2013-2014 The Rust Project Developers.
// Copyright 2018 The Uuid Project Developers.
//
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generate and parse UUIDs.
//!
//! Provides support for Universally Unique Identifiers (UUIDs). A UUID is a
//! unique 128-bit number, stored as 16 octets. UUIDs are used to  assign
//! unique identifiers to entities without requiring a central allocating
//! authority.
//!
//! They are particularly useful in distributed systems, though can be used in
//! disparate areas, such as databases and network protocols.  Typically a UUID
//! is displayed in a readable string form as a sequence of hexadecimal digits,
//! separated into groups by hyphens.
//!
//! The uniqueness property is not strictly guaranteed, however for all
//! practical purposes, it can be assumed that an unintentional collision would
//! be extremely unlikely.
//!
//! # Dependencies
//!
//! By default, this crate depends on nothing but `std` and cannot generate
//! UUIDs. You need to enable the following Cargo features to enable
//! various pieces of functionality:
//!
//! * `v1` - adds the [`Uuid::new_v1`] function and the ability to create a V1
//!   using an implementation of [`v1::ClockSequence`] (usually
//! [`v1::Context`]) and a timestamp from `time::timespec`.
//! * `v3` - adds the [`Uuid::new_v3`] function and the ability to create a V3
//!   UUID based on the MD5 hash of some data.
//! * `v4` - adds the [`Uuid::new_v4`] function and the ability to randomly
//!   generate a UUID.
//! * `v5` - adds the [`Uuid::new_v5`] function and the ability to create a V5
//!   UUID based on the SHA1 hash of some data.
//! * `macros` - adds the `uuid!` macro that can parse UUIDs at compile time.
//! * `serde` - adds the ability to serialize and deserialize a UUID using the
//!   `serde` crate.
//! * `arbitrary` - adds an `Arbitrary` trait implementation to `Uuid`.
//! * `fast-rng` - when combined with `v4` uses a faster algorithm for
//!   generating random UUIDs. This feature requires more dependencies to
//!   compile, but is just as suitable for UUIDs as the default algorithm.
//!
//! By default, `uuid` can be depended on with:
//!
//! ```toml
//! [dependencies]
//! uuid = "0.8"
//! ```
//!
//! To activate various features, use syntax like:
//!
//! ```toml
//! [dependencies]
//! uuid = { version = "0.8", features = ["serde", "v4", "fast-rng"] }
//! ```
//!
//! You can disable default features with:
//!
//! ```toml
//! [dependencies]
//! uuid = { version = "0.8", default-features = false }
//! ```
//!
//! ## Unstable features
//!
//! Some features are unstable. They may be incomplete or depend on other
//! unstable libraries. These include:
//!
//! * `zerocopy-unstable` - adds support for zero-copy deserialization using the
//!   `zerocopy` library.
//!
//! Unstable features may break between minor releases.
//!
//! To allow unstable features, you'll need to enable the Cargo feature as
//! normal, but also pass an additional flag through your environment to opt-in
//! to unstable `uuid` features:
//!
//! ```text
//! RUSTFLAGS="--cfg uuid_unstable"
//! ```
//!
//! # Building for other targets
//!
//! ## WebAssembly
//!
//! For WebAssembly, enable the `js` feature along with `v4` for a
//! source of randomness:
//!
//! ```toml
//! [dependencies]
//! uuid = { version = "0.8", features = ["v4", "js"] }
//! ```
//!
//! You don't need the `js` feature to use `uuid` in WebAssembly if you're
//! not enabling other features too.
//!
//! ## Embedded
//!
//! For embedded targets without the standard library, you'll need to
//! disable default features when building `uuid`:
//!
//! ```toml
//! [dependencies]
//! uuid = { version = "0.8", default-features = false }
//! ```
//!
//! Some additional features are supported in no-std environments:
//!
//! * `v1`, `v3`, and `v5`
//! * `serde`
//!
//! If you need to use `v4` in a no-std environment, you'll need to
//! follow [`getrandom`'s docs] on configuring a source of randomness
//! on currently unsupported targets. Alternatively, you can produce
//! random bytes yourself and then pass them to [`Builder::from_random_bytes`]
//! without enabling the `v4` feature.
//!
//! # Examples
//!
//! To parse a UUID given in the simple format and print it as a urn:
//!
//! ```
//! # use uuid::Uuid;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let my_uuid = Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?;
//!
//! println!("{}", my_uuid.urn());
//! # Ok(())
//! # }
//! ```
//!
//! To create a new random (V4) UUID and print it out in hexadecimal form:
//!
//! ```
//! // Note that this requires the `v4` feature enabled in the uuid crate.
//! # use uuid::Uuid;
//! # fn main() {
//! # #[cfg(feature = "v4")] {
//! let my_uuid = Uuid::new_v4();
//!
//! println!("{}", my_uuid)
//! # }
//! # }
//! ```
//!
//! # References
//!
//! * [Wikipedia: Universally Unique Identifier](http://en.wikipedia.org/wiki/Universally_unique_identifier)
//! * [RFC4122: A Universally Unique IDentifier (UUID) URN Namespace](http://tools.ietf.org/html/rfc4122)
//!
//! [`wasm-bindgen`]: https://crates.io/crates/wasm-bindgen
//! [`cargo-web`]: https://crates.io/crates/cargo-web
//! [`Uuid`]: struct.Uuid.html
//! [`Uuid::new_v1`]: struct.Uuid.html#method.new_v1
//! [`Uuid::new_v3`]: struct.Uuid.html#method.new_v3
//! [`Uuid::new_v4`]: struct.Uuid.html#method.new_v4
//! [`Uuid::new_v5`]: struct.Uuid.html#method.new_v5
//! [`v1::ClockSequence`]: v1/trait.ClockSequence.html
//! [`v1::Context`]: v1/struct.Context.html
//! [`getrandom`'s docs]: https://docs.rs/getrandom

#![no_std]
#![deny(missing_debug_implementations, missing_docs)]
#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://docs.rs/uuid/0.8.1"
)]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[cfg(all(not(feature = "std"), not(test)))]
#[macro_use]
extern crate core as std;

// Check that unstable features are accompanied by a the `uuid_unstable` cfg
#[cfg(all(not(uuid_unstable), feature = "zerocopy-unstable"))]
compile_error!("The `zerocopy-unstable` feature is unstable and may break between releases. Please also pass `RUSTFLAGS=\"--cfg uuid_unstable\"` to allow it.");

#[cfg(feature = "zerocopy-unstable")]
use zerocopy::{AsBytes, FromBytes, Unaligned};

mod builder;
mod error;
mod parser;

pub mod fmt;

#[cfg(feature = "v1")]
pub mod v1;
#[cfg(feature = "v3")]
mod v3;
#[cfg(feature = "v4")]
mod v4;
#[cfg(feature = "v5")]
mod v5;

#[cfg(feature = "rng")]
mod rng;

mod external;

#[cfg(feature = "macros")]
#[macro_use]
mod macros;
#[cfg(feature = "macros")]
pub extern crate uuid_macro;

use crate::std::convert;

pub use crate::{builder::Builder, error::Error};

/// A 128-bit (16 byte) buffer containing the ID.
pub type Bytes = [u8; 16];

/// The version of the UUID, denoting the generating algorithm.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Version {
    /// Special case for `nil` UUID.
    Nil = 0,
    /// Version 1: MAC address.
    Mac,
    /// Version 2: DCE Security.
    Dce,
    /// Version 3: MD5 hash.
    Md5,
    /// Version 4: Random.
    Random,
    /// Version 5: SHA-1 hash.
    Sha1,
}

/// The reserved variants of UUIDs.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Variant {
    /// Reserved by the NCS for backward compatibility.
    NCS = 0,
    /// As described in the RFC4122 Specification (default).
    RFC4122,
    /// Reserved by Microsoft for backward compatibility.
    Microsoft,
    /// Reserved for future expansion.
    Future,
}

/// A Universally Unique Identifier (UUID).
///
/// # Examples
///
/// To parse a UUID given in the simple format and print it as a urn:
///
/// ```
/// # use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let my_uuid = Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?;
///
/// println!("{}", my_uuid.urn());
/// # Ok(())
/// # }
/// ```
///
/// To create a new random (V4) UUID and print it out in hexadecimal form:
///
/// ```
/// // Note that this requires the `v4` feature enabled in the uuid crate.
/// # use uuid::Uuid;
/// # fn main() {
/// # #[cfg(feature = "v4")] {
/// let my_uuid = Uuid::new_v4();
///
/// println!("{}", my_uuid)
/// # }
/// # }
/// ```
///
/// # Formatting
///
/// A UUID can be formatted in one of a few ways:
///
/// * [`simple`](#method.to_simple): `a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8`.
/// * [`hyphenated`](#method.hyphenated):
///   `a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8`.
/// * [`urn`](#method.to_urn): `urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8`.
/// * [`braced`](#method.braced): `{a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8}`.
///
/// The default representation when formatting a UUID with `Display` is
/// hyphenated:
///
/// ```
/// # use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let my_uuid = Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?;
///
/// assert_eq!(
///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
///     my_uuid.to_string(),
/// );
/// # Ok(())
/// # }
/// ```
///
/// Other formats can be specified using adapter methods on the UUID:
///
/// ```
/// # use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let my_uuid = Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?;
///
/// assert_eq!(
///     "urn:uuid:a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
///     my_uuid.urn().to_string(),
/// );
/// # Ok(())
/// # }
/// ```
///
/// # Endianness
///
/// The specification for UUIDs encodes the integer fields that make up the
/// value in big-endian order. This crate assumes integer inputs are already in
/// the correct order by default, regardless of the endianness of the
/// environment. Most methods that accept integers have a `_le` variant (such as
/// `from_fields_le`) that assumes any integer values will need to have their
/// bytes flipped, regardless of the endianness of the environment.
///
/// Most users won't need to worry about endianness unless they need to operate
/// on individual fields (such as when converting between Microsoft GUIDs). The
/// important things to remember are:
///
/// - The endianness is in terms of the fields of the UUID, not the environment.
/// - The endianness is assumed to be big-endian when there's no `_le` suffix
///   somewhere.
/// - Byte-flipping in `_le` methods applies to each integer.
/// - Endianness roundtrips, so if you create a UUID with `from_fields_le`
///   you'll get the same values back out with `to_fields_le`.
///
/// # ABI
///
/// The `Uuid` type is always guaranteed to be have the same ABI as a `[u8;
/// 16]`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "zerocopy-unstable",
    derive(AsBytes, FromBytes, Unaligned)
)]
#[repr(transparent)]
pub struct Uuid(Bytes);

impl Uuid {
    /// UUID namespace for Domain Name System (DNS).
    pub const NAMESPACE_DNS: Self = Uuid([
        0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0,
        0x4f, 0xd4, 0x30, 0xc8,
    ]);

    /// UUID namespace for ISO Object Identifiers (OIDs).
    pub const NAMESPACE_OID: Self = Uuid([
        0x6b, 0xa7, 0xb8, 0x12, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0,
        0x4f, 0xd4, 0x30, 0xc8,
    ]);

    /// UUID namespace for Uniform Resource Locators (URLs).
    pub const NAMESPACE_URL: Self = Uuid([
        0x6b, 0xa7, 0xb8, 0x11, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0,
        0x4f, 0xd4, 0x30, 0xc8,
    ]);

    /// UUID namespace for X.500 Distinguished Names (DNs).
    pub const NAMESPACE_X500: Self = Uuid([
        0x6b, 0xa7, 0xb8, 0x14, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0,
        0x4f, 0xd4, 0x30, 0xc8,
    ]);

    /// Returns the variant of the UUID structure.
    ///
    /// This determines the interpretation of the structure of the UUID.
    /// Currently only the RFC4122 variant is generated by this module.
    /// Callers should only trust the value returned by this method if they
    /// trust the UUID itself.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::{Uuid, Variant};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let my_uuid = Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208")?;
    ///
    /// assert_eq!(Variant::RFC4122, my_uuid.get_variant());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # References
    ///
    /// * [Variant in RFC4122](http://tools.ietf.org/html/rfc4122#section-4.1.1)
    pub const fn get_variant(&self) -> Variant {
        match self.as_bytes()[8] {
            x if x & 0x80 == 0x00 => Variant::NCS,
            x if x & 0xc0 == 0x80 => Variant::RFC4122,
            x if x & 0xe0 == 0xc0 => Variant::Microsoft,
            x if x & 0xe0 == 0xe0 => Variant::Future,
            // The above match arms are actually exhaustive
            // We just return `Future` here because we can't
            // use `unreachable!()` in a `const fn`
            _ => Variant::Future,
        }
    }

    /// Returns the version number of the UUID.
    ///
    /// This represents the algorithm used to generate the contents.
    /// This method is the future-proof alternative to [`Uuid::get_version`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let my_uuid = Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208")?;
    ///
    /// assert_eq!(3, my_uuid.get_version_num());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # References
    ///
    /// * [Version in RFC4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.3)
    pub const fn get_version_num(&self) -> usize {
        (self.as_bytes()[6] >> 4) as usize
    }

    /// Returns the version of the UUID.
    ///
    /// This represents the algorithm used to generate the contents.
    /// If the version field doesn't contain a recognized version then `None`
    /// is returned. If you're trying to read the version for a future extension
    /// you can also use [`Uuid::get_version_num`] to unconditionally return a
    /// number.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::{Uuid, Version};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let my_uuid = Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208")?;
    ///
    /// assert_eq!(Some(Version::Md5), my_uuid.get_version());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # References
    ///
    /// * [Version in RFC4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.3)
    pub const fn get_version(&self) -> Option<Version> {
        match self.get_version_num() {
            0 if self.is_nil() => Some(Version::Nil),
            1 => Some(Version::Mac),
            2 => Some(Version::Dce),
            3 => Some(Version::Md5),
            4 => Some(Version::Random),
            5 => Some(Version::Sha1),
            _ => None,
        }
    }

    /// Returns the four field values of the UUID.
    ///
    /// These values can be passed to the `from_fields()` method to get the
    /// original `Uuid` back.
    ///
    /// * The first field value represents the first group of (eight) hex
    ///   digits, taken as a big-endian `u32` value.  For V1 UUIDs, this field
    ///   represents the low 32 bits of the timestamp.
    /// * The second field value represents the second group of (four) hex
    ///   digits, taken as a big-endian `u16` value.  For V1 UUIDs, this field
    ///   represents the middle 16 bits of the timestamp.
    /// * The third field value represents the third group of (four) hex digits,
    ///   taken as a big-endian `u16` value.  The 4 most significant bits give
    ///   the UUID version, and for V1 UUIDs, the last 12 bits represent the
    ///   high 12 bits of the timestamp.
    /// * The last field value represents the last two groups of four and twelve
    ///   hex digits, taken in order.  The first 1-3 bits of this indicate the
    ///   UUID variant, and for V1 UUIDs, the next 13-15 bits indicate the clock
    ///   sequence and the last 48 bits indicate the node ID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let uuid = Uuid::nil();
    ///
    /// assert_eq!(uuid.as_fields(), (0, 0, 0, &[0u8; 8]));
    ///
    /// let uuid = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")?;
    ///
    /// assert_eq!(
    ///     uuid.as_fields(),
    ///     (
    ///         0xa1a2a3a4,
    ///         0xb1b2,
    ///         0xc1c2,
    ///         &[0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8],
    ///     )
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_fields(&self) -> (u32, u16, u16, &[u8; 8]) {
        let bytes = self.as_bytes();

        let d1 = (bytes[0] as u32) << 24
            | (bytes[1] as u32) << 16
            | (bytes[2] as u32) << 8
            | (bytes[3] as u32);

        let d2 = (bytes[4] as u16) << 8 | (bytes[5] as u16);

        let d3 = (bytes[6] as u16) << 8 | (bytes[7] as u16);

        let d4: &[u8; 8] = convert::TryInto::try_into(&bytes[8..16]).unwrap();
        (d1, d2, d3, d4)
    }

    /// Returns the four field values of the UUID in little-endian order.
    ///
    /// The bytes in the returned integer fields will be converted from
    /// big-endian order. This is based on the endianness of the UUID,
    /// rather than the target environment so bytes will be flipped on both
    /// big and little endian machines.
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let uuid = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")?;
    ///
    /// assert_eq!(
    ///     uuid.to_fields_le(),
    ///     (
    ///         0xa4a3a2a1,
    ///         0xb2b1,
    ///         0xc2c1,
    ///         &[0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8],
    ///     )
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_fields_le(&self) -> (u32, u16, u16, &[u8; 8]) {
        let d1 = (self.as_bytes()[0] as u32)
            | (self.as_bytes()[1] as u32) << 8
            | (self.as_bytes()[2] as u32) << 16
            | (self.as_bytes()[3] as u32) << 24;

        let d2 = (self.as_bytes()[4] as u16) | (self.as_bytes()[5] as u16) << 8;

        let d3 = (self.as_bytes()[6] as u16) | (self.as_bytes()[7] as u16) << 8;

        let d4: &[u8; 8] =
            convert::TryInto::try_into(&self.as_bytes()[8..16]).unwrap();
        (d1, d2, d3, d4)
    }

    /// Returns a 128bit value containing the UUID data.
    ///
    /// The bytes in the UUID will be packed into a `u128`, like the
    /// [`Uuid::as_bytes`] method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let uuid = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")?;
    ///
    /// assert_eq!(
    ///     uuid.as_u128(),
    ///     0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub const fn as_u128(&self) -> u128 {
        (self.as_bytes()[0] as u128) << 120
            | (self.as_bytes()[1] as u128) << 112
            | (self.as_bytes()[2] as u128) << 104
            | (self.as_bytes()[3] as u128) << 96
            | (self.as_bytes()[4] as u128) << 88
            | (self.as_bytes()[5] as u128) << 80
            | (self.as_bytes()[6] as u128) << 72
            | (self.as_bytes()[7] as u128) << 64
            | (self.as_bytes()[8] as u128) << 56
            | (self.as_bytes()[9] as u128) << 48
            | (self.as_bytes()[10] as u128) << 40
            | (self.as_bytes()[11] as u128) << 32
            | (self.as_bytes()[12] as u128) << 24
            | (self.as_bytes()[13] as u128) << 16
            | (self.as_bytes()[14] as u128) << 8
            | (self.as_bytes()[15] as u128)
    }

    /// Returns a 128bit little-endian value containing the UUID data.
    ///
    /// The bytes in the `u128` will be flipped to convert into big-endian
    /// order. This is based on the endianness of the UUID, rather than the
    /// target environment so bytes will be flipped on both big and little
    /// endian machines.
    ///
    /// Note that this will produce a different result than
    /// [`Uuid::to_fields_le`], because the entire UUID is reversed, rather
    /// than reversing the individual fields in-place.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let uuid = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")?;
    ///
    /// assert_eq!(
    ///     uuid.to_u128_le(),
    ///     0xd8d7d6d5d4d3d2d1c2c1b2b1a4a3a2a1,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub const fn to_u128_le(&self) -> u128 {
        (self.as_bytes()[0] as u128)
            | (self.as_bytes()[1] as u128) << 8
            | (self.as_bytes()[2] as u128) << 16
            | (self.as_bytes()[3] as u128) << 24
            | (self.as_bytes()[4] as u128) << 32
            | (self.as_bytes()[5] as u128) << 40
            | (self.as_bytes()[6] as u128) << 48
            | (self.as_bytes()[7] as u128) << 56
            | (self.as_bytes()[8] as u128) << 64
            | (self.as_bytes()[9] as u128) << 72
            | (self.as_bytes()[10] as u128) << 80
            | (self.as_bytes()[11] as u128) << 88
            | (self.as_bytes()[12] as u128) << 96
            | (self.as_bytes()[13] as u128) << 104
            | (self.as_bytes()[14] as u128) << 112
            | (self.as_bytes()[15] as u128) << 120
    }

    /// Returns two 64bit values containing the UUID data.
    ///
    /// The bytes in the UUID will be split into two `u64`.
    /// The first u64 represents the 64 most significant bits,
    /// the second one represents the 64 least significant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let uuid = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")?;
    /// assert_eq!(
    ///     uuid.as_u64_pair(),
    ///     (0xa1a2a3a4b1b2c1c2, 0xd1d2d3d4d5d6d7d8),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub const fn as_u64_pair(&self) -> (u64, u64) {
        let value = self.as_u128();
        ((value >> 64) as u64, value as u64)
    }

    /// Returns an array of 16 octets containing the UUID data.
    pub const fn as_bytes(&self) -> &Bytes {
        &self.0
    }

    /// Tests if the UUID is nil.
    pub const fn is_nil(&self) -> bool {
        self.as_u128() == 0
    }

    /// A buffer that can be used for `encode_...` calls, that is
    /// guaranteed to be long enough for any of the format adapters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let uuid = Uuid::nil();
    ///
    /// assert_eq!(
    ///     uuid.simple().encode_lower(&mut Uuid::encode_buffer()),
    ///     "00000000000000000000000000000000"
    /// );
    ///
    /// assert_eq!(
    ///     uuid.hyphenated()
    ///         .encode_lower(&mut Uuid::encode_buffer()),
    ///     "00000000-0000-0000-0000-000000000000"
    /// );
    ///
    /// assert_eq!(
    ///     uuid.urn().encode_lower(&mut Uuid::encode_buffer()),
    ///     "urn:uuid:00000000-0000-0000-0000-000000000000"
    /// );
    /// ```
    pub const fn encode_buffer() -> [u8; fmt::Urn::LENGTH] {
        [0; fmt::Urn::LENGTH]
    }
}

impl Default for Uuid {
    #[inline]
    fn default() -> Self {
        Uuid::nil()
    }
}

impl AsRef<[u8]> for Uuid {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(feature = "serde")]
pub mod serde {
    //! Adapters for `serde`.
    //!
    //! This module contains adapters you can use with [`#[serde(with)]`](https://serde.rs/field-attrs.html#with)
    //! to change the way a [`Uuid`](../struct.Uuid.html) is serialized
    //! and deserialized.

    pub use crate::external::serde_support::compact;
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::std::string::{String, ToString};

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    macro_rules! check {
        ($buf:ident, $format:expr, $target:expr, $len:expr, $cond:expr) => {
            $buf.clear();
            write!($buf, $format, $target).unwrap();
            assert!($buf.len() == $len);
            assert!($buf.chars().all($cond), "{}", $buf);
        };
    }

    pub const fn new() -> Uuid {
        Uuid::from_bytes([
            0xF9, 0x16, 0x8C, 0x5E, 0xCE, 0xB2, 0x4F, 0xAA, 0xB6, 0xBF, 0x32,
            0x9B, 0xF3, 0x9F, 0xA1, 0xE4,
        ])
    }

    pub const fn new2() -> Uuid {
        Uuid::from_bytes([
            0xF9, 0x16, 0x8C, 0x5E, 0xCE, 0xB2, 0x4F, 0xAB, 0xB6, 0xBF, 0x32,
            0x9B, 0xF3, 0x9F, 0xA1, 0xE4,
        ])
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_uuid_compare() {
        let uuid1 = new();
        let uuid2 = new2();

        assert_eq!(uuid1, uuid1);
        assert_eq!(uuid2, uuid2);

        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid2, uuid1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_uuid_default() {
        let default_uuid = Uuid::default();
        let nil_uuid = Uuid::nil();

        assert_eq!(default_uuid, nil_uuid);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_uuid_display() {
        use crate::std::fmt::Write;

        let uuid = new();
        let s = uuid.to_string();
        let mut buffer = String::new();

        assert_eq!(s, uuid.hyphenated().to_string());

        check!(buffer, "{}", uuid, 36, |c| c.is_lowercase()
            || c.is_digit(10)
            || c == '-');
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_uuid_lowerhex() {
        use crate::std::fmt::Write;

        let mut buffer = String::new();
        let uuid = new();

        check!(buffer, "{:x}", uuid, 36, |c| c.is_lowercase()
            || c.is_digit(10)
            || c == '-');
    }

    // noinspection RsAssertEqual
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_uuid_operator_eq() {
        let uuid1 = new();
        let uuid1_dup = uuid1.clone();
        let uuid2 = new2();

        assert!(uuid1 == uuid1);
        assert!(uuid1 == uuid1_dup);
        assert!(uuid1_dup == uuid1);

        assert!(uuid1 != uuid2);
        assert!(uuid2 != uuid1);
        assert!(uuid1_dup != uuid2);
        assert!(uuid2 != uuid1_dup);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_uuid_to_string() {
        use crate::std::fmt::Write;

        let uuid = new();
        let s = uuid.to_string();
        let mut buffer = String::new();

        assert_eq!(s.len(), 36);

        check!(buffer, "{}", s, 36, |c| c.is_lowercase()
            || c.is_digit(10)
            || c == '-');
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_nil() {
        let nil = Uuid::nil();
        let not_nil = new();
        let from_bytes = Uuid::from_bytes([
            4, 54, 67, 12, 43, 2, 2, 76, 32, 50, 87, 5, 1, 33, 43, 87,
        ]);

        assert_eq!(from_bytes.get_version(), None);

        assert!(nil.is_nil());
        assert!(!not_nil.is_nil());

        assert_eq!(nil.get_version(), Some(Version::Nil));
        assert_eq!(not_nil.get_version(), Some(Version::Random))
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_predefined_namespaces() {
        assert_eq!(
            Uuid::NAMESPACE_DNS.hyphenated().to_string(),
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
        );
        assert_eq!(
            Uuid::NAMESPACE_URL.hyphenated().to_string(),
            "6ba7b811-9dad-11d1-80b4-00c04fd430c8"
        );
        assert_eq!(
            Uuid::NAMESPACE_OID.hyphenated().to_string(),
            "6ba7b812-9dad-11d1-80b4-00c04fd430c8"
        );
        assert_eq!(
            Uuid::NAMESPACE_X500.hyphenated().to_string(),
            "6ba7b814-9dad-11d1-80b4-00c04fd430c8"
        );
    }

    #[cfg(feature = "v3")]
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_get_version_v3() {
        let uuid =
            Uuid::new_v3(&Uuid::NAMESPACE_DNS, "rust-lang.org".as_bytes());

        assert_eq!(uuid.get_version().unwrap(), Version::Md5);
        assert_eq!(uuid.get_version_num(), 3);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_get_variant() {
        let uuid1 = new();
        let uuid2 =
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid3 =
            Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();
        let uuid4 =
            Uuid::parse_str("936DA01F9ABD4d9dC0C702AF85C822A8").unwrap();
        let uuid5 =
            Uuid::parse_str("F9168C5E-CEB2-4faa-D6BF-329BF39FA1E4").unwrap();
        let uuid6 =
            Uuid::parse_str("f81d4fae-7dec-11d0-7765-00a0c91e6bf6").unwrap();

        assert_eq!(uuid1.get_variant(), Variant::RFC4122);
        assert_eq!(uuid2.get_variant(), Variant::RFC4122);
        assert_eq!(uuid3.get_variant(), Variant::RFC4122);
        assert_eq!(uuid4.get_variant(), Variant::Microsoft);
        assert_eq!(uuid5.get_variant(), Variant::Microsoft);
        assert_eq!(uuid6.get_variant(), Variant::NCS);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_to_simple_string() {
        let uuid1 = new();
        let s = uuid1.simple().to_string();

        assert_eq!(s.len(), 32);
        assert!(s.chars().all(|c| c.is_digit(16)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_hyphenated_string() {
        let uuid1 = new();
        let s = uuid1.hyphenated().to_string();

        assert!(s.len() == 36);
        assert!(s.chars().all(|c| c.is_digit(16) || c == '-'));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_upper_lower_hex() {
        use std::fmt::Write;

        let mut buf = String::new();
        let u = new();

        macro_rules! check {
            ($buf:ident, $format:expr, $target:expr, $len:expr, $cond:expr) => {
                $buf.clear();
                write!($buf, $format, $target).unwrap();
                assert_eq!($len, buf.len());
                assert!($buf.chars().all($cond), "{}", $buf);
            };
        }

        check!(buf, "{:x}", u, 36, |c| c.is_lowercase()
            || c.is_digit(10)
            || c == '-');
        check!(buf, "{:X}", u, 36, |c| c.is_uppercase()
            || c.is_digit(10)
            || c == '-');
        check!(buf, "{:#x}", u, 32, |c| c.is_lowercase() || c.is_digit(10));
        check!(buf, "{:#X}", u, 32, |c| c.is_uppercase() || c.is_digit(10));

        check!(buf, "{:X}", u.hyphenated(), 36, |c| c.is_uppercase()
            || c.is_digit(10)
            || c == '-');
        check!(buf, "{:X}", u.simple(), 32, |c| c.is_uppercase()
            || c.is_digit(10));
        check!(buf, "{:#X}", u.hyphenated(), 36, |c| c.is_uppercase()
            || c.is_digit(10)
            || c == '-');
        check!(buf, "{:#X}", u.simple(), 32, |c| c.is_uppercase()
            || c.is_digit(10));

        check!(buf, "{:x}", u.hyphenated(), 36, |c| c.is_lowercase()
            || c.is_digit(10)
            || c == '-');
        check!(buf, "{:x}", u.simple(), 32, |c| c.is_lowercase()
            || c.is_digit(10));
        check!(buf, "{:#x}", u.hyphenated(), 36, |c| c.is_lowercase()
            || c.is_digit(10)
            || c == '-');
        check!(buf, "{:#x}", u.simple(), 32, |c| c.is_lowercase()
            || c.is_digit(10));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_to_urn_string() {
        let uuid1 = new();
        let ss = uuid1.urn().to_string();
        let s = &ss[9..];

        assert!(ss.starts_with("urn:uuid:"));
        assert_eq!(s.len(), 36);
        assert!(s.chars().all(|c| c.is_digit(16) || c == '-'));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_to_simple_string_matching() {
        let uuid1 = new();

        let hs = uuid1.hyphenated().to_string();
        let ss = uuid1.simple().to_string();

        let hsn = hs.chars().filter(|&c| c != '-').collect::<String>();

        assert_eq!(hsn, ss);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_string_roundtrip() {
        let uuid = new();

        let hs = uuid.hyphenated().to_string();
        let uuid_hs = Uuid::parse_str(&hs).unwrap();
        assert_eq!(uuid_hs, uuid);

        let ss = uuid.to_string();
        let uuid_ss = Uuid::parse_str(&ss).unwrap();
        assert_eq!(uuid_ss, uuid);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_from_fields() {
        let d1: u32 = 0xa1a2a3a4;
        let d2: u16 = 0xb1b2;
        let d3: u16 = 0xc1c2;
        let d4 = [0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];

        let u = Uuid::from_fields(d1, d2, d3, &d4);

        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";
        let result = u.simple().to_string();
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_from_fields_le() {
        let d1: u32 = 0xa4a3a2a1;
        let d2: u16 = 0xb2b1;
        let d3: u16 = 0xc2c1;
        let d4 = [0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];

        let u = Uuid::from_fields_le(d1, d2, d3, &d4);

        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";
        let result = u.simple().to_string();
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_as_fields() {
        let u = new();
        let (d1, d2, d3, d4) = u.as_fields();

        assert_ne!(d1, 0);
        assert_ne!(d2, 0);
        assert_ne!(d3, 0);
        assert_eq!(d4.len(), 8);
        assert!(!d4.iter().all(|&b| b == 0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_fields_roundtrip() {
        let d1_in: u32 = 0xa1a2a3a4;
        let d2_in: u16 = 0xb1b2;
        let d3_in: u16 = 0xc1c2;
        let d4_in = &[0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];

        let u = Uuid::from_fields(d1_in, d2_in, d3_in, d4_in);
        let (d1_out, d2_out, d3_out, d4_out) = u.as_fields();

        assert_eq!(d1_in, d1_out);
        assert_eq!(d2_in, d2_out);
        assert_eq!(d3_in, d3_out);
        assert_eq!(d4_in, d4_out);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_fields_le_roundtrip() {
        let d1_in: u32 = 0xa4a3a2a1;
        let d2_in: u16 = 0xb2b1;
        let d3_in: u16 = 0xc2c1;
        let d4_in = &[0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];

        let u = Uuid::from_fields_le(d1_in, d2_in, d3_in, d4_in);
        let (d1_out, d2_out, d3_out, d4_out) = u.to_fields_le();

        assert_eq!(d1_in, d1_out);
        assert_eq!(d2_in, d2_out);
        assert_eq!(d3_in, d3_out);
        assert_eq!(d4_in, d4_out);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_fields_le_are_actually_le() {
        let d1_in: u32 = 0xa1a2a3a4;
        let d2_in: u16 = 0xb1b2;
        let d3_in: u16 = 0xc1c2;
        let d4_in = &[0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];

        let u = Uuid::from_fields(d1_in, d2_in, d3_in, d4_in);
        let (d1_out, d2_out, d3_out, d4_out) = u.to_fields_le();

        assert_eq!(d1_in, d1_out.swap_bytes());
        assert_eq!(d2_in, d2_out.swap_bytes());
        assert_eq!(d3_in, d3_out.swap_bytes());
        assert_eq!(d4_in, d4_out);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_from_u128() {
        let v_in: u128 = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8;

        let u = Uuid::from_u128(v_in);

        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";
        let result = u.simple().to_string();
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_from_u128_le() {
        let v_in: u128 = 0xd8d7d6d5d4d3d2d1c2c1b2b1a4a3a2a1;

        let u = Uuid::from_u128_le(v_in);

        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";
        let result = u.simple().to_string();
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_from_u64_pair() {
        let high_in: u64 = 0xa1a2a3a4b1b2c1c2;
        let low_in: u64 = 0xd1d2d3d4d5d6d7d8;

        let u = Uuid::from_u64_pair(high_in, low_in);

        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";
        let result = u.simple().to_string();
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_u128_roundtrip() {
        let v_in: u128 = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8;

        let u = Uuid::from_u128(v_in);
        let v_out = u.as_u128();

        assert_eq!(v_in, v_out);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_u128_le_roundtrip() {
        let v_in: u128 = 0xd8d7d6d5d4d3d2d1c2c1b2b1a4a3a2a1;

        let u = Uuid::from_u128_le(v_in);
        let v_out = u.to_u128_le();

        assert_eq!(v_in, v_out);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_u64_pair_roundtrip() {
        let high_in: u64 = 0xa1a2a3a4b1b2c1c2;
        let low_in: u64 = 0xd1d2d3d4d5d6d7d8;

        let u = Uuid::from_u64_pair(high_in, low_in);
        let (high_out, low_out) = u.as_u64_pair();

        assert_eq!(high_in, high_out);
        assert_eq!(low_in, low_out);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_u128_le_is_actually_le() {
        let v_in: u128 = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8;

        let u = Uuid::from_u128(v_in);
        let v_out = u.to_u128_le();

        assert_eq!(v_in, v_out.swap_bytes());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_from_slice() {
        let b = [
            0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3,
            0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
        ];

        let u = Uuid::from_slice(&b).unwrap();
        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";

        assert_eq!(u.simple().to_string(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_from_bytes() {
        let b = [
            0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3,
            0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
        ];

        let u = Uuid::from_bytes(b);
        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";

        assert_eq!(u.simple().to_string(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_as_bytes() {
        let u = new();
        let ub = u.as_bytes();
        let ur = u.as_ref();

        assert_eq!(ub.len(), 16);
        assert_eq!(ur.len(), 16);
        assert!(!ub.iter().all(|&b| b == 0));
        assert!(!ur.iter().all(|&b| b == 0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_bytes_roundtrip() {
        let b_in: crate::Bytes = [
            0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3,
            0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
        ];

        let u = Uuid::from_slice(&b_in).unwrap();

        let b_out = u.as_bytes();

        assert_eq!(&b_in, b_out);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_iterbytes_impl_for_uuid() {
        let mut set = std::collections::HashSet::new();
        let id1 = new();
        let id2 = new2();
        set.insert(id1.clone());

        assert!(set.contains(&id1));
        assert!(!set.contains(&id2));
    }
}
