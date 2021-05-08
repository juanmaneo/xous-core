// SPDX-FileCopyrightText: 2020 Sean Cross <sean@xobs.io>
// SPDX-License-Identifier: Apache-2.0

pub const PAGE_SIZE: usize = 4096;
use crate::mem::MemoryManager;
use xous_kernel::{Error, MemoryFlags, PID};

pub const DEFAULT_HEAP_BASE: usize = 0x2000_0000;
pub const DEFAULT_MESSAGE_BASE: usize = 0x4000_0000;
pub const DEFAULT_BASE: usize = 0x6000_0000;

pub const USER_AREA_END: usize = 0xff00_0000;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct MemoryMapping {
    pid: usize,
}

pub const DEFAULT_MEMORY_MAPPING: MemoryMapping = MemoryMapping { pid: 0 };

impl MemoryMapping {
    /// Get the currently active memory mapping.  Note that the actual root pages
    /// may be found at virtual address `PAGE_TABLE_ROOT_OFFSET`.
    pub fn current() -> MemoryMapping {
        MemoryMapping { pid: 0 }
    }

    /// Get the "PID" (actually, ASID) from the current mapping
    pub fn get_pid(self) -> PID {
        unimplemented!()
    }

    /// Set this mapping as the systemwide mapping.
    /// **Note:** This should only be called from an interrupt in the
    /// kernel, which should be mapped into every possible address space.
    /// As such, this will only have an observable effect once code returns
    /// to userspace.
    pub fn activate(self) -> Result<(), xous_kernel::Error> {
        // This is a no-op on hosted environments
        Ok(())
    }

    pub fn reserve_address(
        &mut self,
        _mm: &mut MemoryManager,
        _addr: usize,
        _flags: MemoryFlags,
    ) -> Result<(), Error> {
        Ok(())
    }
}

/// Determine whether a virtual address has been mapped
pub fn address_available(_virt: usize) -> bool {
    true
}

pub fn map_page_inner(
    _mm: &mut MemoryManager,
    _pid: PID,
    _phys: usize,
    _virt: usize,
    _req_flags: MemoryFlags,
    _map_user: bool,
) -> Result<(), xous_kernel::Error> {
    unimplemented!()
}

pub fn move_page_inner(
    _mm: &mut MemoryManager,
    _src_space: &MemoryMapping,
    _src_addr: *mut u8,
    _dest_pid: PID,
    _dest_space: &MemoryMapping,
    _dest_addr: *mut u8,
) -> Result<(), Error> {
    unimplemented!()
}

pub fn lend_page_inner(
    _mm: &mut MemoryManager,
    _src_space: &MemoryMapping,
    _src_addr: *mut u8,
    _dest_pid: PID,
    _dest_space: &MemoryMapping,
    _dest_addr: *mut u8,
    _mutable: bool,
) -> Result<usize, Error> {
    unimplemented!()
}

pub fn return_page_inner(
    _mm: &mut MemoryManager,
    _src_space: &MemoryMapping,
    _src_addr: *mut u8,
    _dest_pid: PID,
    _dest_space: &MemoryMapping,
    _dest_addr: *mut u8,
) -> Result<usize, Error> {
    unimplemented!()
}

pub fn ensure_page_exists_inner(address: usize) -> Result<usize, xous_kernel::Error> {
    unimplemented!()
}

pub fn unmap_page_inner(_mm: &mut MemoryManager, virt: usize) -> Result<usize, Error> {
    Ok(virt)
}

pub fn hand_page_to_user(_virt: *mut u8) -> Result<(), Error> {
    unimplemented!()
}

pub fn virt_to_phys(virt: usize) -> Result<usize, Error> {
    Ok(virt)
}
