/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! helper for FAM (flexible array member) pattern from C

use std::marker::PhantomData;

/// marker struct for a flexible array member
/// (items that extend beyond the end of a struct with a zero size struct in C)
#[repr(C)]
pub struct FAM<T>(PhantomData<T>, [T; 0]);

impl<T> FAM<T> {
    /// gets the data as a pointer
    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self as *const _ as *const T
    }

    /// gets the data as a slice of LENGTH len (not data size, the number of item in the slice)
    #[inline]
    pub unsafe fn flex_ref(&self, len: usize) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), len) }
    }
}
