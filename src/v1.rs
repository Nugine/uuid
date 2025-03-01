//! The implementation for Version 1 UUIDs.
//!
//! Note that you need to enable the `v1` Cargo feature
//! in order to use this module.

use crate::{Uuid, Version};

use atomic::Atomic;

/// The number of 100 ns ticks between the UUID epoch
/// `1582-10-15 00:00:00` and the Unix epoch `1970-01-01 00:00:00`.
const UUID_TICKS_BETWEEN_EPOCHS: u64 = 0x01B2_1DD2_1381_4000;

/// A thread-safe, stateful context for the v1 generator to help ensure
/// process-wide uniqueness.
#[derive(Debug)]
pub struct Context {
    count: Atomic<u16>,
}

/// Stores the number of nanoseconds from an epoch and a counter for ensuring
/// V1 ids generated on the same host are unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp {
    ticks: u64,
    counter: u16,
}

impl Timestamp {
    /// Construct a `Timestamp` from its raw component values: an RFC4122
    /// timestamp and counter.
    ///
    /// RFC4122, which defines the V1 UUID, specifies a 60-byte timestamp format
    /// as the number of 100-nanosecond intervals elapsed since 00:00:00.00,
    /// 15 Oct 1582, "the date of the Gregorian reform of the Christian
    /// calendar."
    ///
    /// The counter value is used to differentiate between ids generated by
    /// the same host computer in rapid succession (i.e. with the same observed
    /// time). See the [`ClockSequence`] trait for a generic interface to any
    /// counter generators that might be used.
    ///
    /// Internally, the timestamp is stored as a `u64`. For this reason, dates
    /// prior to October 1582 are not supported.
    ///
    /// [`ClockSequence`]: trait.ClockSequence.html
    pub const fn from_rfc4122(ticks: u64, counter: u16) -> Self {
        Timestamp { ticks, counter }
    }

    /// Construct a `Timestamp` from a unix timestamp and sequence-generating
    /// `context`.
    ///
    /// A unix timestamp represents the elapsed time since Jan 1 1970. Libc's
    /// `clock_gettime` and other popular implementations traditionally
    /// represent this duration as a `timespec`: a struct with `u64` and
    /// `u32` fields representing the seconds, and "subsecond" or fractional
    /// nanoseconds elapsed since the timestamp's second began,
    /// respectively.
    ///
    /// This constructs a `Timestamp` from the seconds and fractional
    /// nanoseconds of a unix timestamp, converting the duration since 1970
    /// into the number of 100-nanosecond intervals since 00:00:00.00, 15
    /// Oct 1582 specified by RFC4122 and used internally by `Timestamp`.
    ///
    /// The function is not guaranteed to produce monotonically increasing
    /// values however. There is a slight possibility that two successive
    /// equal time values could be supplied and the sequence counter wraps back
    /// over to 0.
    ///
    /// If uniqueness and monotonicity is required, the user is responsible for
    /// ensuring that the time value always increases between calls (including
    /// between restarts of the process and device).
    pub fn from_unix(
        context: impl ClockSequence,
        seconds: u64,
        subsec_nanos: u32,
    ) -> Self {
        let counter = context.generate_sequence(seconds, subsec_nanos);
        let ticks = UUID_TICKS_BETWEEN_EPOCHS
            + seconds * 10_000_000
            + u64::from(subsec_nanos) / 100;

        Timestamp { ticks, counter }
    }

    /// Returns the raw RFC4122 timestamp and counter values stored by the
    /// `Timestamp`.
    ///
    /// The timestamp (the first, `u64` element in the tuple) represents the
    /// number of 100-nanosecond intervals since 00:00:00.00, 15 Oct 1582.
    /// The counter is used to differentiate between ids generated on the
    /// same host computer with the same observed time.
    pub const fn to_rfc4122(&self) -> (u64, u16) {
        (self.ticks, self.counter)
    }

    /// Returns the timestamp converted to the seconds and fractional
    /// nanoseconds since Jan 1 1970.
    ///
    /// Internally, the time is stored in 100-nanosecond intervals,
    /// thus the maximum precision represented by the fractional nanoseconds
    /// value is less than its unit size (100 ns vs. 1 ns).
    pub const fn to_unix(&self) -> (u64, u32) {
        (
            (self.ticks - UUID_TICKS_BETWEEN_EPOCHS) / 10_000_000,
            ((self.ticks - UUID_TICKS_BETWEEN_EPOCHS) % 10_000_000) as u32
                * 100,
        )
    }

    /// Returns the timestamp converted into nanoseconds elapsed since Jan 1
    /// 1970. Internally, the time is stored in 100-nanosecond intervals,
    /// thus the maximum precision represented is less than the units it is
    /// measured in (100 ns vs. 1 ns). The value returned represents the
    /// same duration as [`Timestamp::to_unix`]; this provides it in nanosecond
    /// units for convenience.
    pub const fn to_unix_nanos(&self) -> u64 {
        (self.ticks - UUID_TICKS_BETWEEN_EPOCHS) * 100
    }
}

/// A trait that abstracts over generation of UUID v1 "Clock Sequence" values.
///
/// # References
///
/// * [Clock Sequence in RFC4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.5)
pub trait ClockSequence {
    /// Return a 16-bit number that will be used as the "clock sequence" in
    /// the UUID. The number must be different if the time has changed since
    /// the last time a clock sequence was requested.
    fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> u16;
}

impl<'a, T: ClockSequence + ?Sized> ClockSequence for &'a T {
    fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> u16 {
        (**self).generate_sequence(seconds, subsec_nanos)
    }
}

impl Uuid {
    /// Create a new UUID (version 1) using a time value + sequence +
    /// *NodeId*.
    ///
    /// When generating [`Timestamp`]s using a [`ClockSequence`], this function
    /// is only guaranteed to produce unique values if the following conditions
    /// hold:
    ///
    /// 1. The *NodeId* is unique for this process,
    /// 2. The *Context* is shared across all threads which are generating v1
    ///    UUIDs,
    /// 3. The [`ClockSequence`] implementation reliably returns unique
    ///    clock sequences (this crate provides [`Context`] for this
    ///    purpose. However you can create your own [`ClockSequence`]
    ///    implementation, if [`Context`] does not meet your needs).
    ///
    /// The NodeID must be exactly 6 bytes long.
    ///
    /// Note that usage of this method requires the `v1` feature of this crate
    /// to be enabled.
    ///
    /// # Examples
    ///
    /// A UUID can be created from a unix [`Timestamp`] with a
    /// [`ClockSequence`]. RFC4122 requires the clock sequence
    /// is seeded with a random value:
    ///
    /// ```rust
    /// use uuid::v1::{Timestamp, Context};
    /// # use uuid::Uuid;
    /// # fn random_seed() -> u16 { 42 }
    /// let context = Context::new(random_seed());
    /// let ts = Timestamp::from_unix(&context, 1497624119, 1234);
    ///
    /// let uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "f3b4958c-52a1-11e7-802a-010203040506"
    /// );
    /// ```
    ///
    /// The timestamp can also be created manually as per RFC4122:
    ///
    /// ```
    /// use uuid::v1::{Timestamp, Context};
    /// # use uuid::Uuid;
    ///
    /// let context = Context::new(42);
    /// let ts = Timestamp::from_rfc4122(1497624119, 0);
    ///
    /// let uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "5943ee37-0000-1000-8000-010203040506"
    /// );
    /// ```
    ///
    /// [`Timestamp`]: v1/struct.Timestamp.html
    /// [`ClockSequence`]: v1/trait.ClockSequence.html
    /// [`Context`]: v1/struct.Context.html
    pub const fn new_v1(ts: Timestamp, node_id: &[u8; 6]) -> Self {
        let time_low = (ts.ticks & 0xFFFF_FFFF) as u32;
        let time_mid = ((ts.ticks >> 32) & 0xFFFF) as u16;
        let time_high_and_version =
            (((ts.ticks >> 48) & 0x0FFF) as u16) | (1 << 12);

        let mut d4 = [0; 8];

        d4[0] = (((ts.counter & 0x3F00) >> 8) as u8) | 0x80;
        d4[1] = (ts.counter & 0xFF) as u8;
        d4[2] = node_id[0];
        d4[3] = node_id[1];
        d4[4] = node_id[2];
        d4[5] = node_id[3];
        d4[6] = node_id[4];
        d4[7] = node_id[5];

        Uuid::from_fields(time_low, time_mid, time_high_and_version, &d4)
    }

    /// Returns an optional [`Timestamp`] storing the timestamp and
    /// counter portion parsed from a V1 UUID.
    ///
    /// Returns `None` if the supplied UUID is not V1.
    ///
    /// The V1 timestamp format defined in RFC4122 specifies a 60-bit
    /// integer representing the number of 100-nanosecond intervals
    /// since 00:00:00.00, 15 Oct 1582.
    ///
    /// [`Timestamp`] offers several options for converting the raw RFC4122
    /// value into more commonly-used formats, such as a unix timestamp.
    ///
    /// [`Timestamp`]: v1/struct.Timestamp.html
    pub const fn get_timestamp(&self) -> Option<Timestamp> {
        match self.get_version() {
            Some(Version::Mac) => {
                let ticks: u64 = ((self.as_bytes()[6] & 0x0F) as u64) << 56
                    | ((self.as_bytes()[7]) as u64) << 48
                    | ((self.as_bytes()[4]) as u64) << 40
                    | ((self.as_bytes()[5]) as u64) << 32
                    | ((self.as_bytes()[0]) as u64) << 24
                    | ((self.as_bytes()[1]) as u64) << 16
                    | ((self.as_bytes()[2]) as u64) << 8
                    | (self.as_bytes()[3] as u64);

                let counter: u16 = ((self.as_bytes()[8] & 0x3F) as u16) << 8
                    | (self.as_bytes()[9] as u16);

                Some(Timestamp::from_rfc4122(ticks, counter))
            }
            _ => None,
        }
    }
}

impl Context {
    /// Creates a thread-safe, internally mutable context to help ensure
    /// uniqueness.
    ///
    /// This is a context which can be shared across threads. It maintains an
    /// internal counter that is incremented at every request, the value ends
    /// up in the clock_seq portion of the UUID (the fourth group). This
    /// will improve the probability that the UUID is unique across the
    /// process.
    pub const fn new(count: u16) -> Self {
        Self {
            count: Atomic::new(count),
        }
    }

    /// Creates a thread-safe, internally mutable context that's seeded with a
    /// random value.
    ///
    /// This method requires either the `rng` or `fast-rng` feature to also be
    /// enabled.
    ///
    /// This is a context which can be shared across threads. It maintains an
    /// internal counter that is incremented at every request, the value ends
    /// up in the clock_seq portion of the UUID (the fourth group). This
    /// will improve the probability that the UUID is unique across the
    /// process.
    #[cfg(feature = "rng")]
    pub fn new_random() -> Self {
        Self {
            count: Atomic::new(crate::rng::u16()),
        }
    }
}

impl ClockSequence for Context {
    fn generate_sequence(&self, _: u64, _: u32) -> u16 {
        // RFC4122 reserves 2 bits of the clock sequence so the actual
        // maximum value is smaller than `u16::MAX`. Since we unconditionally
        // increment the clock sequence we want to wrap once it becomes larger
        // than what we can represent in a "u14". Otherwise there'd be patches
        // where the clock sequence doesn't change regardless of the timestamp
        self.count.fetch_add(1, atomic::Ordering::AcqRel) % (u16::MAX >> 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use crate::{std::string::ToString, Variant};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_new_v1() {
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_946_000;
        let node = [1, 2, 3, 4, 5, 6];
        let context = Context::new(0);

        let uuid = Uuid::new_v1(
            Timestamp::from_unix(&context, time, time_fraction),
            &node,
        );

        assert_eq!(uuid.get_version(), Some(Version::Mac));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
        assert_eq!(
            uuid.hyphenated().to_string(),
            "20616934-4ba2-11e7-8000-010203040506"
        );

        let ts = uuid.get_timestamp().unwrap().to_rfc4122();

        assert_eq!(ts.0 - 0x01B2_1DD2_1381_4000, 14_968_545_358_129_460);

        // Ensure parsing the same UUID produces the same timestamp
        let parsed =
            Uuid::parse_str("20616934-4ba2-11e7-8000-010203040506").unwrap();

        assert_eq!(
            uuid.get_timestamp().unwrap(),
            parsed.get_timestamp().unwrap()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_new_v1_context() {
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_946_000;
        let node = [1, 2, 3, 4, 5, 6];

        // This context will wrap
        let context = Context::new((u16::MAX >> 2) - 1);

        let uuid1 = Uuid::new_v1(
            Timestamp::from_unix(&context, time, time_fraction),
            &node,
        );

        let time: u64 = 1_496_854_536;

        let uuid2 = Uuid::new_v1(
            Timestamp::from_unix(&context, time, time_fraction),
            &node,
        );

        assert_eq!(uuid1.get_timestamp().unwrap().to_rfc4122().1, 16382);
        assert_eq!(uuid2.get_timestamp().unwrap().to_rfc4122().1, 0);

        let time = 1_496_854_535;

        let uuid3 = Uuid::new_v1(
            Timestamp::from_unix(&context, time, time_fraction),
            &node,
        );
        let uuid4 = Uuid::new_v1(
            Timestamp::from_unix(&context, time, time_fraction),
            &node,
        );

        assert_eq!(uuid3.get_timestamp().unwrap().to_rfc4122().1, 1);
        assert_eq!(uuid4.get_timestamp().unwrap().to_rfc4122().1, 2);
    }
}
