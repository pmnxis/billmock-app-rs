/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! Experimental const-ish From/Into traits for firmware eco-system
//! Following codes are referenced to rust core's convert codes.

/// Compile-time [`From`] type for const boundary
#[const_trait]
pub trait ConstFrom<T>: Sized {
    /// Converts to this type from the input type.
    fn const_from(value: T) -> Self;
}

/// Compile-time [`Into`] type for const boundary
#[const_trait]
pub trait ConstInto<T>: Sized {
    /// Converts this type into the (usually inferred) input type.
    #[must_use]
    fn const_into(self) -> T;
}

impl<T, U> const ConstInto<U> for T
where
    U: ~const ConstFrom<T> + ~const ConstInto<T>,
{
    /// Calls `U::const_from(self)`.
    ///
    /// That is, this conversion is whatever the implementation of
    /// <code>[ConstFrom]&lt;T&gt; for U</code> chooses to do.
    #[inline]
    fn const_into(self) -> U {
        U::const_from(self)
    }
}

// ConstFrom (and thus ConstInto) is reflexive
impl<T> const ConstFrom<T> for T {
    /// Returns the argument unchanged.
    #[inline(always)]
    fn const_from(t: T) -> T {
        t
    }
}
