/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! generic ioctl helpers

// Size in bits of the ioctl command fields
pub const _IOC_NRBITS: u32 = 8;
pub const _IOC_TYPEBITS: u32 = 8;
pub const _IOC_SIZEBITS: u32 = 14;
pub const _IOC_DIRBITS: u32 = 2;

// Masks for the ioctl command fields
pub const _IOC_NRMASK: u32 = (1 << _IOC_NRBITS) - 1;
pub const _IOC_TYPEMASK: u32 = (1 << _IOC_TYPEBITS) - 1;
pub const _IOC_SIZEMASK: u32 = (1 << _IOC_SIZEBITS) - 1;
pub const _IOC_DIRMASK: u32 = (1 << _IOC_DIRBITS) - 1;

// Bit shifts for the ioctl command fields
pub const _IOC_NRSHIFT: u32 = 0;
pub const _IOC_TYPESHIFT: u32 = _IOC_NRSHIFT + _IOC_NRBITS;
pub const _IOC_SIZESHIFT: u32 = _IOC_TYPESHIFT + _IOC_TYPEBITS;
pub const _IOC_DIRSHIFT: u32 = _IOC_SIZESHIFT + _IOC_SIZEBITS;

// ioctl direction flags
pub const _IOC_NONE: u32 = 0;
pub const _IOC_WRITE: u32 = 1;
pub const _IOC_READ: u32 = 2;

/// dir: direction of the data transfer (read/write)
/// ty: the driver that owns the command (usually an ascii char, 'd' for drm)
/// nr: the command number within the driver's namespace
/// size: size of the arg structure for sanity checking
#[macro_export]
macro_rules! _ioc {
    ($dir:expr, $ty:expr, $nr:expr, $size:expr) => {
        ((($dir) << _IOC_DIRSHIFT)
            | (($ty) << _IOC_TYPESHIFT)
            | (($nr) << _IOC_NRSHIFT)
            | (($size) << _IOC_SIZESHIFT))
    };
}

/// Shorthand for a read-write ioctl command
#[macro_export]
macro_rules! _iowr {
    ($ty:expr, $nr:expr, $argtype:ty) => {
        _ioc!(
            _IOC_READ | _IOC_WRITE,
            $ty,
            $nr,
            size_of::<$argtype>() as u32
        )
    };
}
