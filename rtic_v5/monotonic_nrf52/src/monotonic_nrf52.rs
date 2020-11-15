//! Using NRF52 as monotonic timer

use core::u32;
use core::{
    cmp::Ordering,
    convert::{Infallible, TryInto},
    fmt, ops,
};
use nrf52832_hal::target;
use rtic::Monotonic;

/// A measurement of the counter. Opaque and useful only with `Duration`
///
/// # Correctness
///
/// Adding or subtracting a `Duration` of more than `(1 << 31)` cycles to an `Instant` effectively
/// makes it "wrap around" and creates an incorrect value. This is also true if the operation is
/// done in steps, e.g. `(instant + dur) + dur` where `dur` is `(1 << 30)` ticks.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Instant {
    inner: i32,
}

impl Instant {
    /// Returns an instant corresponding to "now"
    pub fn now() -> Self {
        let now = {
            let timer = unsafe { &*target::TIMER1::ptr() };
            timer.tasks_capture[0].write(|w| unsafe { w.bits(1) });
            timer.cc[0].read().bits()
        };

        Instant { inner: now as i32 }
    }

    /// Returns the amount of time elapsed since this instant was created.
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    /// Returns the underlying count
    pub fn counts(&self) -> u32 {
        self.inner as u32
    }

    /// Returns the amount of time elapsed from another instant to this one.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let diff = self.inner.wrapping_sub(earlier.inner);
        assert!(diff >= 0, "second instant is later than self");
        Duration { inner: diff as u32 }
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Instant")
            .field(&(self.inner as u32))
            .finish()
    }
}

impl ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, dur: Duration) {
        // NOTE this is a debug assertion because there's no foolproof way to detect a wrap around;
        // the user may write `(instant + dur) + dur` where `dur` is `(1<<31)-1` ticks.
        debug_assert!(dur.inner < (1 << 31));
        self.inner = self.inner.wrapping_add(dur.inner as i32);
    }
}

impl ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(mut self, dur: Duration) -> Self {
        self += dur;
        self
    }
}

impl ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, dur: Duration) {
        // NOTE see the NOTE in `<Instant as AddAssign<Duration>>::add_assign`
        debug_assert!(dur.inner < (1 << 31));
        self.inner = self.inner.wrapping_sub(dur.inner as i32);
    }
}

impl ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(mut self, dur: Duration) -> Self {
        self -= dur;
        self
    }
}

impl ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.inner.wrapping_sub(rhs.inner).cmp(&0)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

/// A `Duration` type to represent a span of time.
///
/// This data type is only available on ARMv7-M
///
/// # Correctness
///
/// This type is *not* appropriate for representing time spans in the order of, or larger than,
/// seconds because it can hold a maximum of `(1 << 31)` "ticks" where each tick is the inverse of
/// the CPU frequency, which usually is dozens of MHz.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Duration {
    inner: u32,
}

impl Duration {
    /// Creates a new `Duration` from the specified number of clock cycles
    pub fn from_cycles(cycles: u32) -> Self {
        Duration { inner: cycles }
    }

    /// Returns the total number of clock cycles contained by this `Duration`
    pub fn as_cycles(&self) -> u32 {
        self.inner
    }
}

// Used internally by RTIC to convert the duration into a known type
impl TryInto<u32> for Duration {
    type Error = Infallible;

    fn try_into(self) -> Result<u32, Infallible> {
        Ok(self.as_cycles())
    }
}

impl ops::AddAssign for Duration {
    fn add_assign(&mut self, dur: Duration) {
        self.inner += dur.inner;
    }
}

impl ops::Add<Duration> for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Duration {
            inner: self.inner + other.inner,
        }
    }
}

impl ops::Mul<u32> for Duration {
    type Output = Self;

    fn mul(self, other: u32) -> Self {
        Duration {
            inner: self.inner * other,
        }
    }
}

impl ops::MulAssign<u32> for Duration {
    fn mul_assign(&mut self, other: u32) {
        *self = *self * other;
    }
}

impl ops::SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        self.inner -= rhs.inner;
    }
}

impl ops::Sub<Duration> for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Duration {
            inner: self.inner - rhs.inner,
        }
    }
}

/// Adds the `secs`, `millis` and `micros` methods to the `u32` type
///
/// This trait is only available on ARMv7-M
pub trait U32Ext {
    /// Converts the `u32` value as seconds into ticks
    fn secs(self) -> Duration;

    /// Converts the `u32` value as milliseconds into ticks
    fn millis(self) -> Duration;

    /// Converts the `u32` value as microseconds into ticks
    fn micros(self) -> Duration;
}

impl U32Ext for u32 {
    fn secs(self) -> Duration {
        self.millis() * 1_000
    }

    fn millis(self) -> Duration {
        self.micros() * 1_000
    }

    fn micros(self) -> Duration {
        let frac = Tim1::ratio();

        // 64 MHz / fraction / 1_000_000
        Duration {
            inner: (64 * frac.denominator * self) / frac.numerator,
        }
    }
}

/// Implementor of the `rtic::Monotonic` traits and used to "eat" the timer to not allow for
/// erroneous configuration
pub struct Tim1;

impl Tim1 {
    pub fn initialize(timer: target::TIMER1) {
        // Auto restart, make sure the entire timer won't stop for any event
        timer.shorts.write(|w| {
            w.compare0_clear()
                .enabled()
                .compare0_stop()
                .disabled()
                .compare1_clear()
                .enabled()
                .compare1_stop()
                .disabled()
                .compare2_clear()
                .enabled()
                .compare2_stop()
                .disabled()
                .compare3_clear()
                .enabled()
                .compare3_stop()
                .disabled()
                .compare4_clear()
                .enabled()
                .compare4_stop()
                .disabled()
                .compare5_clear()
                .enabled()
                .compare5_stop()
                .disabled()
        });

        // 1 MHz mode
        timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });

        // 32 bit mode
        timer.bitmode.write(|w| w.bitmode()._32bit());

        // Set compare value to max, not sure if this is needed
        timer.cc[0].write(|w| unsafe { w.cc().bits(u32::MAX) });

        // Clear the counter value
        timer.tasks_clear.write(|w| unsafe { w.bits(1) });

        // Start the timer
        timer.tasks_start.write(|w| unsafe { w.bits(1) });

        // Throw away the timer, it is now setup and consumed
        drop(timer);
    }
}

impl rtic::Monotonic for Tim1 {
    type Instant = Instant;

    fn ratio() -> rtic::Fraction {
        // monotonic * fraction = sys clock
        rtic::Fraction {
            numerator: 64,
            denominator: 1,
        }
    }

    fn now() -> Self::Instant {
        Instant::now()
    }

    unsafe fn reset() {
        let timer = &*target::TIMER1::ptr();

        // Clear the counter value
        timer.tasks_clear.write(|w| w.bits(1));
    }

    fn zero() -> Self::Instant {
        Instant { inner: 0 }
    }
}
