//! Memory management implementation
//!
//! This module implements memory management including permissions,
//! memory mapping, and memory access operations.

use crate::EmulatorError;
use std::collections::BTreeMap;

/// Memory permissions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Permissions {
    /// No access
    None,
    /// Read only
    Read,
    /// Read and write
    ReadWrite,
    /// Read and execute
    ReadExecute,
    /// Read, write, and execute
    ReadWriteExecute,
}

impl Permissions {
    /// Check if permission contains read access
    pub fn can_read(&self) -> bool {
        matches!(
            self,
            Self::Read | Self::ReadWrite | Self::ReadExecute | Self::ReadWriteExecute
        )
    }

    /// Check if permission contains write access
    pub fn can_write(&self) -> bool {
        matches!(self, Self::ReadWrite | Self::ReadWriteExecute)
    }

    /// Check if permission contains execute access
    pub fn can_execute(&self) -> bool {
        matches!(self, Self::ReadExecute | Self::ReadWriteExecute)
    }

    /// Check if permission contains another permission
    pub fn contains(&self, other: Permissions) -> bool {
        match (self, other) {
            (_, Permissions::None) => true,
            (Permissions::None, _) => false,
            (Permissions::Read, Permissions::Read) => true,
            (Permissions::ReadWrite, Permissions::Read | Permissions::ReadWrite) => true,
            (Permissions::ReadExecute, Permissions::Read | Permissions::ReadExecute) => true,
            (Permissions::ReadWriteExecute, _) => true,
            _ => false,
        }
    }
}

/// Memory region
#[derive(Debug)]
struct Region {
    /// Base address
    base: u64,
    /// Size in bytes
    size: u64,
    /// Access permissions
    permissions: Permissions,
    /// Memory contents
    data: Vec<u8>,
}

/// Cache line state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheLineState {
    /// Invalid
    Invalid,
    /// Modified (dirty)
    Modified,
    /// Exclusive
    Exclusive,
    /// Shared
    Shared,
}

/// Cache line
#[derive(Debug)]
struct CacheLine {
    /// Tag
    tag: u64,
    /// Data
    data: Vec<u8>,
    /// State
    state: CacheLineState,
    /// Last access time for LRU
    last_access: u64,
}

impl CacheLine {
    fn new(tag: u64, size: usize) -> Self {
        Self {
            tag,
            data: vec![0; size],
            state: CacheLineState::Invalid,
            last_access: 0,
        }
    }

    /// Check if line is dirty (modified)
    fn is_dirty(&self) -> bool {
        self.state == CacheLineState::Modified
    }

    /// Write back dirty line to memory
    fn write_back(&mut self, memory: &mut Memory, addr: u64) -> Result<(), EmulatorError> {
        if self.is_dirty() {
            memory.write_bytes(addr, &self.data)?;
            self.state = CacheLineState::Exclusive;
        }
        Ok(())
    }
}

/// Cache set
#[derive(Debug)]
struct CacheSet {
    /// Lines in the set
    lines: Vec<CacheLine>,
    /// Access counter for LRU
    access_counter: u64,
    /// Set index
    index: usize,
}

impl CacheSet {
    fn new(index: usize, associativity: usize, line_size: usize) -> Self {
        Self {
            lines: (0..associativity)
                .map(|_| CacheLine::new(0, line_size))
                .collect(),
            access_counter: 0,
            index,
        }
    }

    fn find_line(&mut self, tag: u64) -> Option<&mut CacheLine> {
        self.access_counter += 1;
        for line in &mut self.lines {
            if line.state != CacheLineState::Invalid && line.tag == tag {
                line.last_access = self.access_counter;
                return Some(line);
            }
        }
        None
    }

    fn find_victim(&mut self) -> usize {
        self.access_counter += 1;

        // First try to find an invalid line
        if let Some((idx, _)) = self
            .lines
            .iter()
            .enumerate()
            .find(|(_, l)| l.state == CacheLineState::Invalid)
        {
            return idx;
        }

        // Otherwise use LRU replacement
        self.lines
            .iter()
            .enumerate()
            .min_by_key(|(_, line)| line.last_access)
            .map(|(i, _)| i)
            .unwrap()
    }

    fn find_line_mut(&mut self, tag: u64) -> Option<&mut CacheLine> {
        self.lines.iter_mut().find(|line| line.tag == tag)
    }
}

/// Cache write policy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WritePolicy {
    /// Write-through: writes go to all levels immediately
    WriteThrough,
    /// Write-back: writes update cache only, dirty lines written back later
    WriteBack,
}

/// Cache hint
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheHint {
    /// Normal caching
    Normal,
    /// Non-temporal (bypass L1)
    NonTemporal1,
    /// Non-temporal (bypass all caches)
    NonTemporalAll,
    /// Bias toward keeping in cache
    Bias,
}

/// Cache level
#[derive(Debug)]
struct CacheLevel {
    /// Sets in the cache
    sets: Vec<CacheSet>,
    /// Number of sets
    num_sets: usize,
    /// Set associativity
    associativity: usize,
    /// Line size in bytes
    line_size: usize,
    /// Line size bits for address decomposition
    line_bits: u32,
    /// Set bits for address decomposition
    set_bits: u32,
    /// Non-temporal hint active
    non_temporal: bool,
    /// Write policy
    write_policy: WritePolicy,
}

impl CacheLevel {
    fn new(size: usize, associativity: usize, line_size: usize) -> Self {
        let num_sets = size / (associativity * line_size);
        let line_bits = line_size.trailing_zeros();
        let set_bits = num_sets.trailing_zeros();

        Self {
            sets: (0..num_sets)
                .map(|i| CacheSet::new(i, associativity, line_size))
                .collect(),
            num_sets,
            associativity,
            line_size,
            line_bits,
            set_bits,
            non_temporal: false,
            write_policy: WritePolicy::WriteThrough,
        }
    }

    fn decompose_address(&self, addr: u64) -> (u64, usize, usize) {
        let offset = addr & ((1 << self.line_bits) - 1);
        let set_idx = ((addr >> self.line_bits) & ((1 << self.set_bits) - 1)) as usize;
        let tag = addr >> (self.line_bits + self.set_bits);
        (tag, set_idx, offset as usize)
    }

    fn compose_address(&self, tag: u64, set_idx: usize) -> u64 {
        (tag << (self.line_bits + self.set_bits)) | ((set_idx as u64) << self.line_bits)
    }

    fn get_set_index(&self, addr: u64) -> usize {
        let (_, set_idx, _) = self.decompose_address(addr);
        set_idx
    }

    fn get_tag(&self, addr: u64) -> u64 {
        let (tag, _, _) = self.decompose_address(addr);
        tag
    }

    fn get_offset(&self, addr: u64) -> usize {
        let (_, _, offset) = self.decompose_address(addr);
        offset
    }

    fn read(&mut self, addr: u64, data: &mut [u8]) -> bool {
        if self.non_temporal {
            return false;
        }

        let (tag, set_idx, offset) = self.decompose_address(addr);
        let set = &mut self.sets[set_idx];

        if let Some(line) = set.find_line(tag) {
            // Cache hit
            data.copy_from_slice(&line.data[offset..offset + data.len()]);
            true
        } else {
            false
        }
    }

    /// Write data to cache
    fn write(&mut self, addr: u64, data: &[u8]) {
        let (old_addr, old_data) = self.write_to_cache(addr, data);
        if let Some(old_data) = old_data {
            // Write back to memory will be handled by the caller
            // This avoids the need for a mutable reference to Memory
            // and simplifies the borrowing rules
        }
    }

    /// Flush cache to memory
    fn flush(&mut self) -> Vec<(u64, Vec<u8>)> {
        let mut dirty_lines = Vec::new();
        let mut set_indices = Vec::new();
        let mut tags = Vec::new();

        // First collect all the information we need
        for (set_idx, set) in self.sets.iter_mut().enumerate() {
            for line in &mut set.lines {
                if line.state == CacheLineState::Modified {
                    set_indices.push(set_idx);
                    tags.push(line.tag);
                    dirty_lines.push(line.data.clone());
                    line.state = CacheLineState::Exclusive;
                }
            }
        }

        // Now compose addresses
        dirty_lines
            .into_iter()
            .zip(tags)
            .zip(set_indices)
            .map(|((data, tag), set_idx)| {
                let addr = self.compose_address(tag, set_idx);
                (addr, data)
            })
            .collect()
    }

    fn set_non_temporal(&mut self, value: bool) {
        self.non_temporal = value;
    }

    fn write_to_cache(&mut self, addr: u64, data: &[u8]) -> (u64, Option<Vec<u8>>) {
        let (tag, set_index, offset) = self.decompose_address(addr);
        let set = &mut self.sets[set_index];
        let counter = set.access_counter;

        if let Some(line) = set.find_line_mut(tag) {
            // Cache hit
            line.data[offset..offset + data.len()].copy_from_slice(data);
            line.state = CacheLineState::Modified;
            line.last_access = counter;
            set.access_counter += 1;
            return (0, None); // No eviction needed
        }

        // Cache miss - need to find a line to evict
        let victim_idx = set.find_victim();
        let victim = &mut set.lines[victim_idx];
        let old_tag = victim.tag;
        let old_data = if victim.state == CacheLineState::Modified {
            Some(victim.data.clone())
        } else {
            None
        };

        // Update the victim line
        victim.tag = tag;
        victim.data[offset..offset + data.len()].copy_from_slice(data);
        victim.state = CacheLineState::Modified;
        victim.last_access = counter;
        set.access_counter += 1;

        let old_addr = self.compose_address(old_tag, set_index);
        (old_addr, old_data)
    }
}

/// Memory management unit
#[derive(Debug)]
pub struct Memory {
    /// Memory regions
    regions: BTreeMap<u64, Region>,
    /// L1 cache
    l1_cache: CacheLevel,
    /// L2 cache
    l2_cache: CacheLevel,
    /// L3 cache
    l3_cache: CacheLevel,
    /// Speculative loads
    speculative_loads: Vec<SpeculativeLoad>,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    /// Create new memory instance
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            // 32KB L1 cache, 8-way associative, 64-byte lines
            l1_cache: CacheLevel::new(32 * 1024, 8, 64),
            // 256KB L2 cache, 8-way associative, 64-byte lines
            l2_cache: CacheLevel::new(256 * 1024, 8, 64),
            // 6MB L3 cache, 12-way associative, 128-byte lines
            l3_cache: CacheLevel::new(6 * 1024 * 1024, 12, 128),
            speculative_loads: Vec::new(),
        }
    }

    /// Set cache hints
    pub fn set_cache_hints(&mut self, hint: CacheHint) {
        match hint {
            CacheHint::NonTemporal1 => {
                self.l1_cache.set_non_temporal(true);
                self.l2_cache.set_non_temporal(false);
                self.l3_cache.set_non_temporal(false);
            }
            CacheHint::NonTemporalAll => {
                self.l1_cache.set_non_temporal(true);
                self.l2_cache.set_non_temporal(true);
                self.l3_cache.set_non_temporal(true);
            }
            CacheHint::Bias => {
                // For bias hint, we keep data in all cache levels
                self.l1_cache.set_non_temporal(false);
                self.l2_cache.set_non_temporal(false);
                self.l3_cache.set_non_temporal(false);
            }
            CacheHint::Normal => {
                self.l1_cache.set_non_temporal(false);
                self.l2_cache.set_non_temporal(false);
                self.l3_cache.set_non_temporal(false);
            }
        }
    }

    /// Map memory region
    pub fn map(
        &mut self,
        base: u64,
        size: u64,
        permissions: Permissions,
    ) -> Result<(), EmulatorError> {
        // Check for overlapping regions
        for (_, region) in self.regions.range(..=base) {
            if region.base + region.size > base {
                return Err(EmulatorError::MemoryError(
                    "Overlapping memory region".to_string(),
                ));
            }
        }

        let region = Region {
            base,
            size,
            permissions,
            data: vec![0; size as usize],
        };

        self.regions.insert(base, region);
        Ok(())
    }

    /// Unmap memory region
    pub fn unmap(&mut self, base: u64) -> Result<(), EmulatorError> {
        if self.regions.remove(&base).is_none() {
            return Err(EmulatorError::MemoryError("Region not found".to_string()));
        }
        Ok(())
    }

    /// Read byte from memory with caching
    pub fn read_u8(&mut self, addr: u64) -> Result<u8, EmulatorError> {
        // Check permissions first
        let region = self.find_region(addr)?;
        if !region.permissions.can_read() {
            return Err(EmulatorError::MemoryError(
                "Read permission denied".to_string(),
            ));
        }

        let offset = (addr - region.base) as usize;
        let memory_data = region.data[offset];
        let _ = region; // Release the region borrow

        let mut data = [0u8; 1];

        // Try L1 cache first
        if !self.l1_cache.non_temporal && self.l1_cache.read(addr, &mut data) {
            return Ok(data[0]);
        }

        // L1 miss, try L2
        if !self.l2_cache.non_temporal && self.l2_cache.read(addr, &mut data) {
            // Fill L1 if not non-temporal
            if !self.l1_cache.non_temporal {
                self.l1_cache.write_to_cache(addr, &[data[0]]);
            }
            return Ok(data[0]);
        }

        // L2 miss, try L3
        if !self.l3_cache.non_temporal && self.l3_cache.read(addr, &mut data) {
            // Fill L2 if not non-temporal
            if !self.l2_cache.non_temporal {
                self.l2_cache.write_to_cache(addr, &[data[0]]);
            }
            // Fill L1 if not non-temporal
            if !self.l1_cache.non_temporal {
                self.l1_cache.write_to_cache(addr, &[data[0]]);
            }
            return Ok(data[0]);
        }

        // Cache miss or non-temporal access, use memory data
        let data = memory_data;

        // Fill L3 if not non-temporal
        if !self.l3_cache.non_temporal {
            self.l3_cache.write_to_cache(addr, &[data]);

            // Fill L2 if not non-temporal
            if !self.l2_cache.non_temporal {
                self.l2_cache.write_to_cache(addr, &[data]);

                // Fill L1 if not non-temporal
                if !self.l1_cache.non_temporal {
                    self.l1_cache.write_to_cache(addr, &[data]);
                }
            }
        }

        Ok(data)
    }

    /// Write byte to memory with caching
    pub fn write_u8(&mut self, addr: u64, value: u8) -> Result<(), EmulatorError> {
        self.write_to_caches(addr, &[value])
    }

    /// Read 64-bit value from memory
    pub fn read_u64(&mut self, addr: u64) -> Result<u64, EmulatorError> {
        let mut value = 0u64;
        for i in 0..8 {
            value |= (self.read_u8(addr + i)? as u64) << (i * 8);
        }
        Ok(value)
    }

    /// Write 64-bit value to memory
    pub fn write_u64(&mut self, addr: u64, value: u64) -> Result<(), EmulatorError> {
        let mut data = [0u8; 8];
        for i in 0..8 {
            data[i] = ((value >> (i * 8)) & 0xFF) as u8;
        }
        self.write_to_caches(addr, &data)
    }

    /// Read 16-bit value from memory
    pub fn read_u16(&mut self, addr: u64) -> Result<u16, EmulatorError> {
        let mut value = 0u16;
        for i in 0..2 {
            value |= (self.read_u8(addr + i)? as u16) << (i * 8);
        }
        Ok(value)
    }

    /// Read 32-bit value from memory
    pub fn read_u32(&mut self, addr: u64) -> Result<u32, EmulatorError> {
        let mut value = 0u32;
        for i in 0..4 {
            value |= (self.read_u8(addr + i)? as u32) << (i * 8);
        }
        Ok(value)
    }

    /// Write 16-bit value to memory
    pub fn write_u16(&mut self, addr: u64, value: u16) -> Result<(), EmulatorError> {
        for i in 0..2 {
            self.write_u8(addr + i, ((value >> (i * 8)) & 0xFF) as u8)?;
        }
        Ok(())
    }

    /// Write 32-bit value to memory
    pub fn write_u32(&mut self, addr: u64, value: u32) -> Result<(), EmulatorError> {
        for i in 0..4 {
            self.write_u8(addr + i, ((value >> (i * 8)) & 0xFF) as u8)?;
        }
        Ok(())
    }

    /// Memory fence operation
    pub fn fence(&mut self) -> Result<(), EmulatorError> {
        // Memory fence ensures all previous memory operations are complete
        // before subsequent operations begin. In our emulator, this is a no-op
        // since we execute instructions sequentially.
        Ok(())
    }

    /// Find memory region containing address
    fn find_region(&self, addr: u64) -> Result<&Region, EmulatorError> {
        let (_, region) = self
            .regions
            .range(..=addr)
            .next_back()
            .ok_or_else(|| EmulatorError::MemoryError("Address not mapped".to_string()))?;

        if addr >= region.base + region.size {
            return Err(EmulatorError::MemoryError("Address not mapped".to_string()));
        }

        Ok(region)
    }

    /// Find mutable memory region containing address
    fn find_region_mut(&mut self, addr: u64) -> Result<&mut Region, EmulatorError> {
        let base = self.find_region(addr)?.base;
        Ok(self.regions.get_mut(&base).unwrap())
    }

    /// Track a speculative load
    pub fn track_speculative_load(
        &mut self,
        addr: u64,
        size: usize,
    ) -> Result<SpeculativeStatus, EmulatorError> {
        // Try to perform the load
        let mut data = vec![0; size];
        match self.read_bytes(addr, &mut data) {
            Ok(_) => {
                // Load succeeded - track it
                let load = SpeculativeLoad {
                    addr,
                    size,
                    status: SpeculativeStatus::Success,
                    data,
                };
                self.speculative_loads.push(load);
                Ok(SpeculativeStatus::Success)
            }
            Err(e) => {
                // Load failed - track failure
                let load = SpeculativeLoad {
                    addr,
                    size,
                    status: SpeculativeStatus::Failed,
                    data: vec![],
                };
                self.speculative_loads.push(load);
                Ok(SpeculativeStatus::Failed)
            }
        }
    }

    /// Cancel a speculative load
    pub fn cancel_speculative_load(&mut self, addr: u64) {
        if let Some(load) = self.speculative_loads.iter_mut().find(|l| l.addr == addr) {
            load.status = SpeculativeStatus::Cancelled;
        }
    }

    /// Check if a speculative load succeeded
    pub fn check_speculative_load(&self, addr: u64) -> Option<SpeculativeStatus> {
        self.speculative_loads
            .iter()
            .find(|l| l.addr == addr)
            .map(|l| l.status)
    }

    /// Read bytes from memory
    pub fn read_bytes(&mut self, addr: u64, data: &mut [u8]) -> Result<(), EmulatorError> {
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = self.read_u8(addr + i as u64)?;
        }
        Ok(())
    }

    /// Write bytes to memory
    pub fn write_bytes(&mut self, addr: u64, data: &[u8]) -> Result<(), EmulatorError> {
        for (i, &byte) in data.iter().enumerate() {
            self.write_u8(addr + i as u64, byte)?;
        }
        Ok(())
    }

    fn write_to_caches(&mut self, addr: u64, data: &[u8]) -> Result<(), EmulatorError> {
        // Check permissions first
        let region = self.find_region(addr)?;
        if !region.permissions.can_write() {
            return Err(EmulatorError::MemoryError(
                "Write permission denied".to_string(),
            ));
        }

        // Check if write would exceed region bounds
        let offset = (addr - region.base) as usize;
        if offset + data.len() > region.size as usize {
            return Err(EmulatorError::MemoryError(
                "Write exceeds region bounds".to_string(),
            ));
        }

        // Cache the non-temporal flags before borrowing self
        let l3_temporal = !self.l3_cache.non_temporal;
        let l2_temporal = !self.l2_cache.non_temporal;
        let l1_temporal = !self.l1_cache.non_temporal;

        // Write to memory first
        let region = self.find_region_mut(addr)?;
        region.data[offset..offset + data.len()].copy_from_slice(data);

        // Then update caches if not non-temporal
        if l3_temporal {
            let (l3_old_addr, l3_old_data) = self.l3_cache.write_to_cache(addr, data);
            if let Some(l3_data) = l3_old_data {
                let region = self.find_region_mut(l3_old_addr)?;
                let offset = (l3_old_addr - region.base) as usize;
                if offset + l3_data.len() <= region.size as usize {
                    region.data[offset..offset + l3_data.len()].copy_from_slice(&l3_data);
                }
            }

            if l2_temporal {
                let (l2_old_addr, l2_old_data) = self.l2_cache.write_to_cache(addr, data);
                if let Some(l2_data) = l2_old_data {
                    let region = self.find_region_mut(l2_old_addr)?;
                    let offset = (l2_old_addr - region.base) as usize;
                    if offset + l2_data.len() <= region.size as usize {
                        region.data[offset..offset + l2_data.len()].copy_from_slice(&l2_data);
                    }
                }

                if l1_temporal {
                    let (l1_old_addr, l1_old_data) = self.l1_cache.write_to_cache(addr, data);
                    if let Some(l1_data) = l1_old_data {
                        let region = self.find_region_mut(l1_old_addr)?;
                        let offset = (l1_old_addr - region.base) as usize;
                        if offset + l1_data.len() <= region.size as usize {
                            region.data[offset..offset + l1_data.len()].copy_from_slice(&l1_data);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), EmulatorError> {
        self.write_to_caches(addr, data)
    }

    fn flush_cache(&mut self, level: &mut CacheLevel) -> Result<(), EmulatorError> {
        let dirty_lines = level.flush();
        for (addr, data) in dirty_lines {
            let region = self.find_region_mut(addr)?;
            let offset = (addr - region.base) as usize;
            region.data[offset..offset + data.len()].copy_from_slice(&data);
        }
        Ok(())
    }

    /// Flush all cache levels back to memory
    ///
    /// This operation ensures that any modified data in any cache level is written back to main memory.
    /// After this operation completes, all cache lines will be in the Exclusive state.
    ///
    /// # Returns
    /// - `Ok(())` if all cache lines were successfully flushed
    /// - `Err(EmulatorError)` if there was an error writing back to memory
    pub fn flush_all_caches(&mut self) -> Result<(), EmulatorError> {
        // First collect all dirty lines
        let l1_dirty = self.l1_cache.flush();
        let l2_dirty = self.l2_cache.flush();
        let l3_dirty = self.l3_cache.flush();

        // Write back all dirty lines
        for (addr, data) in l1_dirty
            .into_iter()
            .chain(l2_dirty.into_iter())
            .chain(l3_dirty.into_iter())
        {
            let region = self.find_region_mut(addr)?;
            let offset = (addr - region.base) as usize;
            region.data[offset..offset + data.len()].copy_from_slice(&data);
        }

        Ok(())
    }
}

/// Speculative load status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpeculativeStatus {
    /// Load succeeded
    Success,
    /// Load failed
    Failed,
    /// Load was cancelled
    Cancelled,
}

/// Speculative load entry
#[derive(Debug)]
struct SpeculativeLoad {
    /// Memory address
    addr: u64,
    /// Size in bytes
    size: usize,
    /// Status of the load
    status: SpeculativeStatus,
    /// Data loaded (if successful)
    data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions() {
        let perm = Permissions::ReadWriteExecute;
        assert!(perm.can_read());
        assert!(perm.can_write());
        assert!(perm.can_execute());

        let perm = Permissions::ReadWrite;
        assert!(perm.can_read());
        assert!(perm.can_write());
        assert!(!perm.can_execute());

        let perm = Permissions::Read;
        assert!(perm.can_read());
        assert!(!perm.can_write());
        assert!(!perm.can_execute());
    }

    #[test]
    fn test_memory_mapping() {
        let mut mem = Memory::new();

        // Map region
        assert!(mem.map(0x1000, 0x1000, Permissions::ReadWrite).is_ok());

        // Try to map overlapping region
        assert!(mem.map(0x1800, 0x1000, Permissions::ReadWrite).is_err());

        // Unmap region
        assert!(mem.unmap(0x1000).is_ok());

        // Try to unmap non-existent region
        assert!(mem.unmap(0x2000).is_err());
    }

    #[test]
    fn test_memory_access() {
        let mut mem = Memory::new();
        mem.map(0x1000, 0x1000, Permissions::ReadWrite).unwrap();

        // Write and read byte
        mem.write_u8(0x1000, 0x42).unwrap();
        assert_eq!(mem.read_u8(0x1000).unwrap(), 0x42);

        // Write and read u64
        mem.write_u64(0x1008, 0x0123456789ABCDEF).unwrap();
        assert_eq!(mem.read_u64(0x1008).unwrap(), 0x0123456789ABCDEF);

        // Try to access unmapped memory
        assert!(mem.read_u8(0x2000).is_err());
        assert!(mem.write_u8(0x2000, 0x42).is_err());
    }

    #[test]
    fn test_memory_permissions() {
        let mut mem = Memory::new();

        // Read-only memory
        mem.map(0x1000, 0x1000, Permissions::Read).unwrap();
        assert!(mem.read_u8(0x1000).is_ok());
        assert!(mem.write_u8(0x1000, 0x42).is_err());

        // No access memory
        mem.map(0x2000, 0x1000, Permissions::None).unwrap();
        assert!(mem.read_u8(0x2000).is_err());
        assert!(mem.write_u8(0x2000, 0x42).is_err());
    }

    #[test]
    fn test_memory_boundaries() {
        let mut mem = Memory::new();
        mem.map(0x1000, 0x1000, Permissions::ReadWrite).unwrap();

        // Access at region boundaries
        assert!(mem.read_u8(0x1000).is_ok());
        assert!(mem.read_u8(0x1FFF).is_ok());
        assert!(mem.read_u8(0x2000).is_err());

        // Write u64 at region boundary should fail
        assert!(mem.write_u64(0x1FF9, 0x42).is_err());
    }

    #[test]
    fn test_cache_hints() {
        let mut mem = Memory::new();
        mem.map(0x1000, 4096, Permissions::ReadWriteExecute)
            .unwrap();

        // Write some data
        mem.write_u8(0x1000, 0x42).unwrap();

        // Test normal caching
        mem.set_cache_hints(CacheHint::Normal);
        let val = mem.read_u8(0x1000).unwrap();
        assert_eq!(val, 0x42);

        // Test L1 bypass
        mem.set_cache_hints(CacheHint::NonTemporal1);
        let val = mem.read_u8(0x1000).unwrap();
        assert_eq!(val, 0x42);

        // Test all cache bypass
        mem.set_cache_hints(CacheHint::NonTemporalAll);
        let val = mem.read_u8(0x1000).unwrap();
        assert_eq!(val, 0x42);

        // Test cache bias
        mem.set_cache_hints(CacheHint::Bias);
        let val = mem.read_u8(0x1000).unwrap();
        assert_eq!(val, 0x42);
    }

    #[test]
    fn test_write_back_caching() {
        let mut mem = Memory::new();
        mem.map(0x1000, 4096, Permissions::ReadWriteExecute)
            .unwrap();

        // Configure L1 cache as write-back
        mem.l1_cache.write_policy = WritePolicy::WriteBack;

        // Write data to cache
        mem.write_u64(0x1000, 0x1234567890ABCDEF).unwrap();

        // Data should be in L1 cache but not in memory yet
        let cache_line = mem.l1_cache.sets[0]
            .lines
            .iter()
            .find(|line| line.state == CacheLineState::Modified)
            .unwrap();
        assert!(cache_line.is_dirty());

        // Reading should hit the cache
        assert_eq!(mem.read_u64(0x1000).unwrap(), 0x1234567890ABCDEF);

        // Flush cache
        mem.flush_all_caches().unwrap();

        // Cache line should no longer be dirty
        let cache_line = mem.l1_cache.sets[0]
            .lines
            .iter()
            .find(|line| line.state == CacheLineState::Exclusive)
            .unwrap();
        assert!(!cache_line.is_dirty());

        // Data should still be readable from memory
        assert_eq!(mem.read_u64(0x1000).unwrap(), 0x1234567890ABCDEF);
    }

    #[test]
    fn test_write_back_eviction() {
        let mut mem = Memory::new();
        mem.map(0x1000, 4096, Permissions::ReadWriteExecute)
            .unwrap();

        // Configure L1 cache as write-back
        mem.l1_cache.write_policy = WritePolicy::WriteBack;

        // Fill cache set with data
        for i in 0..8 {
            // L1 is 8-way associative
            mem.write_u64(0x1000 + i * 64, i as u64).unwrap(); // Each cache line is 64 bytes
        }

        // Write one more value to cause eviction
        mem.write_u64(0x1000 + 8 * 64, 8).unwrap();

        // First value should have been written back to memory
        assert_eq!(mem.read_u64(0x1000).unwrap(), 0);
    }

    #[test]
    fn test_speculative_loads() {
        let mut mem = Memory::new();
        mem.map(0x1000, 4096, Permissions::ReadWriteExecute)
            .unwrap();

        // Write some test data
        mem.write_u64(0x1000, 0x1234567890ABCDEF).unwrap();

        // Track successful speculative load
        let status = mem.track_speculative_load(0x1000, 8).unwrap();
        assert_eq!(status, SpeculativeStatus::Success);

        // Check load status
        assert_eq!(
            mem.check_speculative_load(0x1000),
            Some(SpeculativeStatus::Success)
        );

        // Cancel a load
        mem.cancel_speculative_load(0x1000);
        assert_eq!(
            mem.check_speculative_load(0x1000),
            Some(SpeculativeStatus::Cancelled)
        );

        // Track failed load (unmapped memory)
        let status = mem.track_speculative_load(0x2000, 8).unwrap();
        assert_eq!(status, SpeculativeStatus::Failed);
        assert_eq!(
            mem.check_speculative_load(0x2000),
            Some(SpeculativeStatus::Failed)
        );

        // Check non-existent load
        assert_eq!(mem.check_speculative_load(0x3000), None);
    }
}
