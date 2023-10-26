#![allow(dead_code, unused_imports)]

use std::convert::TryInto;
use std::ops::{Add, AddAssign, Sub, SubAssign};
pub use std::time::*;
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(std::time::SystemTime);

#[cfg(not(target_arch = "wasm32"))]
impl SystemTime {
    pub fn now() -> Self {
        Self(std::time::SystemTime::now())
    }
    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        self.0.duration_since(earlier.0)
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub const UNIX_EPOCH: SystemTime = SystemTime(std::time::SystemTime::UNIX_EPOCH);

// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen(inline_js = r#"
// export function performance_now() {
//   return performance.now();
// }"#)]
// extern "C" {
//     fn performance_now() -> f64;
// }

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(u64);

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTimeError;

#[cfg(target_arch = "wasm32")]
impl SystemTime {
    pub fn now() -> Self {
        Self((js_sys::Date::now() * 1000.0) as u64)
    }
    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        Ok(Duration::from_micros(self.0 - earlier.0))
    }
}
#[cfg(target_arch = "wasm32")]
pub const UNIX_EPOCH: SystemTime = SystemTime(0);

// impl Add<Duration> for Instant {
//     type Output = Instant;
//     fn add(self, other: Duration) -> Instant {
//         self.checked_add(other).unwrap()
//     }
// }
// impl Sub<Duration> for Instant {
//     type Output = Instant;
//     fn sub(self, other: Duration) -> Instant {
//         self.checked_sub(other).unwrap()
//     }
// }
// impl Sub<Instant> for Instant {
//     type Output = Duration;
//     fn sub(self, other: Instant) -> Duration {
//         self.duration_since(other)
//     }
// }
// impl AddAssign<Duration> for Instant {
//     fn add_assign(&mut self, other: Duration) {
//         *self = *self + other;
//     }
// }
// impl SubAssign<Duration> for Instant {
//     fn sub_assign(&mut self, other: Duration) {
//         *self = *self - other;
//     }
// }
