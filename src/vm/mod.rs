use crate::memory::Chunk;

#[derive(Copy, Clone, Debug)]
pub struct PhysicalAddress(pub usize);

impl core::ops::Add<usize> for PhysicalAddress {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        PhysicalAddress(self.0 + rhs)
    }
}

impl core::ops::Sub<PhysicalAddress> for PhysicalAddress {
    type Output = usize;
    fn sub(self, rhs: PhysicalAddress) -> Self::Output {
        self.0 - rhs.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VirtualAddress(pub usize);

impl core::ops::Add<usize> for VirtualAddress {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        VirtualAddress(self.0 + rhs)
    }
}

impl core::ops::AddAssign<usize> for VirtualAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub<VirtualAddress> for VirtualAddress {
    type Output = usize;
    fn sub(self, rhs: VirtualAddress) -> Self::Output {
        self.0 - rhs.0
    }
}

pub trait Table {
    /// Returns Err(()) and doesn't map anything if any virtual address in this range is already
    /// mapped
    //TODO: remove result, add return value to unmap(), call unmap() to check at all call sites
    fn map_to(
        &mut self,
        virt: VirtualAddress,
        phys: PhysicalAddress,
        size: usize,
    ) -> Result<(), ()>;

    fn unmap(&mut self, virt: VirtualAddress, size: usize);

    fn clear(&mut self);

    fn alloc(&mut self, mut virt: VirtualAddress, mut size: usize) -> Result<(), ()> {
        self.unmap(virt, size);
        let mut frame_alloc = crate::memory::FRAME_ALLOCATOR.lock();
        while size > 0 {
            let Chunk { phys: chunk_phys, size: chunk_size } = frame_alloc.alloc_range(size);
            size -= chunk_size;
            virt += chunk_size;
            self.map_to(virt, chunk_phys, size)?;
        }
        Ok(())
    }
}
