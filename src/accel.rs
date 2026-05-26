//! Hardware-accelerated bit gather / scatter (PEXT / PDEP).
//!
//! **This is the only file in the crate that uses `unsafe`.** Every other
//! source file is covered by `#![deny(unsafe_code)]`. The unsafe blocks
//! here are gated by `#[cfg(target_feature = "bmi2")]` for compile-time
//! enablement, or by `std::is_x86_feature_detected!("bmi2")` when the
//! `runtime-detect` feature is active.
//!
//! Algorithms:
//!
//! * `pext(x, mask)` — for each bit set in `mask` from low to high, take
//!   the corresponding bit of `x` and pack it into the low bits of the
//!   result. (Hardware: BMI2 `PEXT`.)
//! * `pdep(x, mask)` — take the low `popcount(mask)` bits of `x` and
//!   scatter them into the positions set in `mask`. Inverse of `pext`
//!   on the same mask. (Hardware: BMI2 `PDEP`.)
//!
//! Fallbacks: the portable SWAR implementations are *correct on every
//! target*, just slower (≈ 30 cycles vs 1 cycle on capable Intel/AMD).

#![allow(unsafe_code)]

// ---- u32 -----------------------------------------------------------------

#[inline]
pub(crate) fn pext_u32(x: u32, mask: u32) -> u32 {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    {
        // SAFETY: target_feature guarantees BMI2 is present on this build target.
        return unsafe { core::arch::x86_64::_pext_u32(x, mask) };
    }
    #[cfg(all(
        target_arch = "x86_64",
        not(target_feature = "bmi2"),
        feature = "runtime-detect",
    ))]
    {
        if std::is_x86_feature_detected!("bmi2") {
            // SAFETY: runtime check confirmed BMI2 is present.
            return unsafe { core::arch::x86_64::_pext_u32(x, mask) };
        }
    }
    pext_u32_swar(x, mask)
}

#[inline]
pub(crate) fn pdep_u32(x: u32, mask: u32) -> u32 {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    {
        // SAFETY: target_feature guarantees BMI2 is present.
        return unsafe { core::arch::x86_64::_pdep_u32(x, mask) };
    }
    #[cfg(all(
        target_arch = "x86_64",
        not(target_feature = "bmi2"),
        feature = "runtime-detect",
    ))]
    {
        if std::is_x86_feature_detected!("bmi2") {
            // SAFETY: runtime check confirmed BMI2 is present.
            return unsafe { core::arch::x86_64::_pdep_u32(x, mask) };
        }
    }
    pdep_u32_swar(x, mask)
}

/// Portable PEXT for `u32`. Equivalent to BMI2 `PEXT` semantically.
#[inline]
pub(crate) const fn pext_u32_swar(x: u32, mut mask: u32) -> u32 {
    let mut res = 0u32;
    let mut bb = 1u32;
    while mask != 0 {
        let lowest = mask & mask.wrapping_neg();
        if (x & lowest) != 0 { res |= bb; }
        mask ^= lowest;
        bb <<= 1;
    }
    res
}

/// Portable PDEP for `u32`. Equivalent to BMI2 `PDEP` semantically.
#[inline]
pub(crate) const fn pdep_u32_swar(x: u32, mut mask: u32) -> u32 {
    let mut res = 0u32;
    let mut bb = 1u32;
    while mask != 0 {
        let lowest = mask & mask.wrapping_neg();
        if (x & bb) != 0 { res |= lowest; }
        mask ^= lowest;
        bb <<= 1;
    }
    res
}

// ---- u64 -----------------------------------------------------------------

#[inline]
pub(crate) fn pext_u64(x: u64, mask: u64) -> u64 {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    {
        // SAFETY: target_feature guarantees BMI2.
        return unsafe { core::arch::x86_64::_pext_u64(x, mask) };
    }
    #[cfg(all(
        target_arch = "x86_64",
        not(target_feature = "bmi2"),
        feature = "runtime-detect",
    ))]
    {
        if std::is_x86_feature_detected!("bmi2") {
            // SAFETY: runtime-checked BMI2.
            return unsafe { core::arch::x86_64::_pext_u64(x, mask) };
        }
    }
    pext_u64_swar(x, mask)
}

#[inline]
pub(crate) fn pdep_u64(x: u64, mask: u64) -> u64 {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    {
        // SAFETY: target_feature guarantees BMI2.
        return unsafe { core::arch::x86_64::_pdep_u64(x, mask) };
    }
    #[cfg(all(
        target_arch = "x86_64",
        not(target_feature = "bmi2"),
        feature = "runtime-detect",
    ))]
    {
        if std::is_x86_feature_detected!("bmi2") {
            // SAFETY: runtime-checked BMI2.
            return unsafe { core::arch::x86_64::_pdep_u64(x, mask) };
        }
    }
    pdep_u64_swar(x, mask)
}

/// Portable PEXT for `u64`.
#[inline]
pub(crate) const fn pext_u64_swar(x: u64, mut mask: u64) -> u64 {
    let mut res = 0u64;
    let mut bb = 1u64;
    while mask != 0 {
        let lowest = mask & mask.wrapping_neg();
        if (x & lowest) != 0 { res |= bb; }
        mask ^= lowest;
        bb <<= 1;
    }
    res
}

/// Portable PDEP for `u64`.
#[inline]
pub(crate) const fn pdep_u64_swar(x: u64, mut mask: u64) -> u64 {
    let mut res = 0u64;
    let mut bb = 1u64;
    while mask != 0 {
        let lowest = mask & mask.wrapping_neg();
        if (x & bb) != 0 { res |= lowest; }
        mask ^= lowest;
        bb <<= 1;
    }
    res
}
