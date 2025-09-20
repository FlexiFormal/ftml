#![allow(clippy::cast_precision_loss)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

use std::fmt::Display;
use std::num::NonZeroU64;
use std::ops::Mul;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;

pub fn measure<R>(f: impl FnOnce() -> R) -> (R, Delta) {
    let now = Timestamp::now();
    let r = f();
    let delta = now.since_now();
    (r, delta)
}

pub async fn measure_async<R, F: Future<Output = R>>(f: impl FnOnce() -> F) -> (R, Delta) {
    let now = Timestamp::now();
    let r = f().await;
    let delta = now.since_now();
    (r, delta)
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Timestamp(#[cfg_attr(feature = "typescript", tsify(type = "number"))] pub NonZeroU64);

trait AsU64 {
    fn into_u64(self) -> u64;
}
impl AsU64 for u64 {
    #[inline]
    fn into_u64(self) -> u64 {
        self
    }
}
impl AsU64 for i64 {
    #[inline]
    fn into_u64(self) -> u64 {
        self as u64
    }
}

#[inline]
fn non_zero<I: AsU64>(i: I) -> NonZeroU64 {
    // SAFETY 1 != 0
    NonZeroU64::new(i.into_u64()).unwrap_or(unsafe { NonZeroU64::new_unchecked(1) })
}
impl Timestamp {
    #[must_use]
    #[inline]
    pub fn now() -> Self {
        let t = chrono::Utc::now().timestamp_millis();
        Self(non_zero(t))
    }

    #[must_use]
    #[inline]
    pub fn zero() -> Self {
        Self(non_zero(1u64))
    }

    #[must_use]
    pub fn since_now(self) -> Delta {
        let t = self.0.get();
        let n = chrono::Utc::now().timestamp_millis() as u64;
        Delta(non_zero(n - t))
    }
    #[must_use]
    pub fn since(self, o: Self) -> Delta {
        let o = o.0.get();
        let s = self.0.get();
        Delta(non_zero(s - o))
    }

    #[must_use]
    pub const fn into_date(self) -> Date {
        Date(self.0)
    }
}
impl Default for Timestamp {
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}
impl From<std::time::SystemTime> for Timestamp {
    #[inline]
    fn from(t: std::time::SystemTime) -> Self {
        let t = t
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| unreachable!());
        Self(non_zero(t.as_millis() as u64))
    }
}
impl FromStr for Timestamp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = chrono::DateTime::<chrono::Utc>::from_str(s).map_err(|_| ())?;
        Ok(Self(non_zero(c.timestamp_millis())))
    }
}
impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts = self.0.get() as i64;
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ts)
            .unwrap_or_else(|| unreachable!())
            .with_timezone(&chrono::Local);
        timestamp.format("%Y-%m-%d %H:%M:%S").fmt(f)
    }
}

pub struct AtomicTimestamp(pub AtomicU64);
impl AtomicTimestamp {
    #[must_use]
    #[inline]
    pub fn now() -> Self {
        let t = chrono::Utc::now().timestamp_millis();
        Self(AtomicU64::new(t as u64))
    }

    pub fn update(&self) -> Timestamp {
        Timestamp(non_zero(self.0.swap(
            Timestamp::now().0.get(),
            std::sync::atomic::Ordering::AcqRel,
        )))
    }

    #[must_use]
    pub fn since_now(&self) -> Delta {
        let t = self.0.load(std::sync::atomic::Ordering::Acquire);
        let n = chrono::Utc::now().timestamp_millis() as u64;
        Delta(non_zero(n - t))
    }

    #[must_use]
    pub fn since(&self, o: &Self) -> Delta {
        let o = o.0.load(std::sync::atomic::Ordering::Acquire);
        let s = self.0.load(std::sync::atomic::Ordering::Acquire);
        Delta(non_zero(s - o))
    }

    #[must_use]
    pub fn as_date(&self) -> Date {
        Date(non_zero(self.0.load(std::sync::atomic::Ordering::Acquire)))
    }
}
impl Default for AtomicTimestamp {
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

pub struct Date(NonZeroU64);
impl From<Timestamp> for Date {
    #[inline]
    fn from(t: Timestamp) -> Self {
        Self(t.0)
    }
}
impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts = self.0.get() as i64;
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ts)
            .unwrap_or_else(|| unreachable!())
            .with_timezone(&chrono::Local);
        timestamp.format("%Y-%m-%d").fmt(f)
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Delta(NonZeroU64);
impl Default for Delta {
    fn default() -> Self {
        Self(unsafe { NonZeroU64::new_unchecked(1) })
    }
}
impl Delta {
    #[must_use]
    pub fn max_seconds(self) -> impl Display {
        MaxSeconds(self)
    }
    pub fn update_average(&mut self, scale: f64, new: Self) {
        let old = self.0.get() as f64;
        if old == 1.0 {
            self.0 = new.0;
            return;
        }
        let new = new.0.get() as f64;
        let t = scale.mul_add(old, (1.0 - scale) * new) as u64;
        self.0 = non_zero(t);
    }
    #[must_use]
    pub fn step_second(self) -> Self {
        let t = self.0.get();
        if t > 1000 {
            Self(non_zero(t - 1000))
        } else {
            Self::default()
        }
    }
}
impl Mul<f64> for Delta {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let t = (self.0.get() as f64 * rhs) as u64;
        Self(non_zero(t))
    }
}
impl Delta {
    pub const SECOND: Self = Self(NonZeroU64::new(1000).unwrap());
    pub const MINUTE: Self = Self(NonZeroU64::new(60_000).unwrap());
    pub const HOUR: Self = Self(NonZeroU64::new(3_600_000).unwrap());
    pub const DAY: Self = Self(NonZeroU64::new(86_400_000).unwrap());
    pub const WEEK: Self = Self(NonZeroU64::new(604_800_000).unwrap());
    pub const MONTH: Self = Self(NonZeroU64::new(2_592_000_000).unwrap());
    pub const YEAR: Self = Self(NonZeroU64::new(31_536_000_000).unwrap());
}
impl Display for Delta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = chrono::Duration::milliseconds(self.0.get() as i64);
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;
        let seconds = duration.num_seconds() % 60;
        let millis = duration.num_milliseconds() % 1000;

        let mut is_empty = true;

        if days > 0 {
            is_empty = false;
            f.write_str(&format!("{days:02}d "))?;
        }
        if hours > 0 {
            is_empty = false;
            f.write_str(&format!("{hours:02}h "))?;
        }
        if minutes > 0 {
            is_empty = false;
            f.write_str(&format!("{minutes:02}m "))?;
        }
        if seconds > 0 {
            is_empty = false;
            f.write_str(&format!("{seconds:02}s "))?;
        }
        if millis > 0 {
            is_empty = false;
            f.write_str(&format!("{millis:02}ms"))?;
        }
        if is_empty {
            f.write_str("0ms")?;
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MaxSeconds(Delta);
impl Display for MaxSeconds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = chrono::Duration::milliseconds(self.0.0.get() as i64);
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;
        let seconds = duration.num_seconds() % 60;

        let mut is_empty = true;

        if days > 0 {
            is_empty = false;
            f.write_str(&format!("{days:02}d "))?;
        }
        if hours > 0 {
            is_empty = false;
            f.write_str(&format!("{hours:02}h "))?;
        }
        if minutes > 0 {
            is_empty = false;
            f.write_str(&format!("{minutes:02}m "))?;
        }
        if seconds > 0 {
            is_empty = false;
            f.write_str(&format!("{seconds:02}s "))?;
        }
        if is_empty {
            f.write_str("0s")?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eta {
    pub time_left: Delta,
    pub done: usize,
    pub total: usize,
}
