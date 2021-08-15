// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Secure string type
//!
//! A `SecureString` cannot be cloned and will wipe its memory when dropped.
//!
//! It is possible to leak the `SecureString`'s data, notably by dereferencing
//! it to a standard [String] then cloning it.

use std::{
    ops::{Deref, Drop},
    fmt,
};


/// A String type that cannot be cloned and securely erases its data when
/// deallocated.
///
/// It is possible to leak a `SecureString` in a number of ways; we do not
/// provide strong guarantees that the data will not exist anywhere on the
/// system, even after dropping it.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecureString(String);

impl fmt::Display for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        unsafe { secure_erase(&mut self.0.as_mut_vec()) }
    }
}

/// Dereference a `SecureString` to its underlying [String].
///
/// Note that we cannot prevent the dereferenced string from being cloned.
// TODO: Should I create an `as_cloneable_ref()` method instead?
impl Deref for SecureString {
    type Target = String;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self { Self(s) }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self { Self(String::from(s)) }
}

/// Clear a memory slice. The optimizer will not optimize this operation away.
///
/// This operation will be slower than similar operations provided by the
/// operating system; if the destination OS is known, those functions should be
/// preferred, especially for large objects.
///
/// # Aborts
///
/// `secure_erase()` will abort if the slice is not aligned.
///
/// # Safety
///
/// The safety requirements of [std::ptr::write_volatile] must be maintained.
// TODO: Use OS/non-standard libc calls when present?
pub unsafe fn secure_erase(mut slice: impl AsMut<[u8]>) {
    use std::ptr::write_volatile;

    let range = slice.as_mut().as_mut_ptr_range();
    let mut ptr = range.start;

    while ptr < range.end {
        write_volatile(ptr, 0);
        ptr = ptr.add(1);
    }
}
