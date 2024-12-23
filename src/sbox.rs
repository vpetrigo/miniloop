//! # `StackBox` implementation
//!
//! A module for working with `StackBox`, a container designed to store pinned references to values
//! on the stack.
//!
//! This module provides functionality for safely pinning values in place, which is essential for
//! certain types such as `Future`s, generators, or other types that depend on stable memory
//! addresses.
//!
//! The `StackBox` struct ensures that values are pinned in place on the stack, avoiding unnecessary
//! heap allocation while maintaining safety and ensuring values cannot be moved out of the pinned
//! context. Additionally, it provides a convenient type alias for working with stack-pinned
//! `Future`s (`StackBoxFuture`).
//!
//! # Features
//! - `StackBox` for safely wrapping and pinning stack-based values.
//! - Type alias `StackBoxFuture` for stack-based pinned trait objects implementing `Future`.

use crate::task::TaskFuture;

use core::cell::OnceCell;
use core::pin::Pin;

/// A container for holding a pinned reference to a value on the stack.
///
/// The `StackBox` struct provides a way to safely pin a value in place on the stack.
/// A pinned reference means that the value pointed to by the reference cannot be moved.
/// This is important for certain types that rely on stable addresses, such as generators or futures.
///
/// # Type Parameters
/// - `'a`: The lifetime of the reference to the stored value.
/// - `T`: The type of the value to be stored. The type may be dynamically sized (`?Sized`).
pub struct StackBox<'a, T: ?Sized> {
    /// A `OnceCell` containing a pinned mutable reference to the stored value.
    pub value: OnceCell<Pin<&'a mut T>>,
}

impl<T: ?Sized> Default for StackBox<'_, T> {
    fn default() -> Self {
        StackBox {
            value: OnceCell::new(),
        }
    }
}

impl<'a, T: ?Sized> StackBox<'a, T> {
    /// Creates a new `StackBox` containing a pinned reference to the provided value.
    ///
    /// # Arguments
    /// - `value`: A mutable reference to the value to be stored. The reference must have the
    ///   appropriate lifetime `'a`.
    ///
    /// # Returns
    /// A `StackBox` containing a pinned mutable reference to the provided value.
    ///
    /// # Safety
    /// This function uses `Pin::new_unchecked`, which is unsafe because it assumes
    /// that the value being pinned will not move for the duration of the pin.
    /// Ensure that the value cannot be moved out of the `StackBox`.
    pub fn new(value: &'a mut T) -> Self {
        let new_box = StackBox::default();
        new_box
            .value
            .get_or_init(|| unsafe { Pin::new_unchecked(value) });

        new_box
    }
}

/// A type alias for a `StackBox` containing a `Future` trait object.
///
/// The `StackBoxFuture` type is a convenient way to create a stack-based pinned
/// future. This allows futures to be stored and run on the stack rather than
/// being allocated on the heap, which can be useful in certain performance-sensitive
/// scenarios.
///
/// # Type Parameters
/// - `'a`: The lifetime of the reference to the stored future.
pub type StackBoxFuture<'a> = StackBox<'a, dyn TaskFuture + 'a>;
