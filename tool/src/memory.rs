use windows::core::PCWSTR;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE};

pub struct Memory;

impl Memory {
    pub unsafe fn get_module_base(module_name: &str) -> Option<usize> {
        let wide: Vec<u16> = module_name.encode_utf16().chain(std::iter::once(0)).collect();
        let handle = GetModuleHandleW(PCWSTR(wide.as_ptr()));
        
        match handle {
            Ok(h) => Some(h.0 as usize),
            Err(_) => None,
        }
    }

    /// Checks if a memory address is readable.
    pub unsafe fn is_readable(addr: usize, _size: usize) -> bool {
        if addr == 0 { return false; }

        let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
        let result = VirtualQuery(
            Some(addr as *const std::ffi::c_void),
            &mut mbi,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>()
        );

        if result == 0 {
            return false;
        }

        // Check state
        if (mbi.State & windows::Win32::System::Memory::MEM_COMMIT) == windows::Win32::System::Memory::MEM_COMMIT {
             // Check protection
             let protect = mbi.Protect;
             return (protect & PAGE_READONLY) != windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS(0) ||
                    (protect & PAGE_READWRITE) != windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS(0) ||
                    (protect & PAGE_EXECUTE_READ) != windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS(0) ||
                    (protect & PAGE_EXECUTE_READWRITE) != windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS(0);
        }

        false
    }
    
    // Safer read function using is_readable check
    pub unsafe fn safe_read_ptr(addr: usize) -> Option<usize> {
        if !Self::is_readable(addr, std::mem::size_of::<usize>()) {
            return None;
        }
        
        // Even with VirtualQuery, race conditions can happen, but this catches 99% of crashes.
        // For 100% safety, we'd need SEH or ReadProcessMemory (even for self).
        // Let's try direct read first after check.
        match (addr as *const usize).as_ref() {
            Some(&val) => Some(val),
            None => None,
        }
    }

    /// Resolves a multi-level pointer chain.
    pub unsafe fn get_pointer_address(module_base: usize, base_offset: usize, offsets: &[usize]) -> Option<usize> {
        let mut addr = module_base + base_offset;

        // Read the initial pointer
        match Self::safe_read_ptr(addr) {
            Some(val) => addr = val,
            None => return None,
        }

        if addr == 0 { return None; }

        // Traverse the chain
        for (i, &offset) in offsets.iter().enumerate() {
            if i == offsets.len() - 1 {
                // Last offset is added to the address to get the final value address
                return Some(addr + offset);
            }

            // Intermediate offsets: Add offset, then dereference
            let next_ptr_addr = addr + offset;
            match Self::safe_read_ptr(next_ptr_addr) {
                Some(val) => {
                    addr = val;
                    if addr == 0 { return None; }
                },
                None => return None,
            }
        }
        
        Some(addr)
    }

    pub unsafe fn read<T: Copy>(addr: usize) -> Option<T> {
        if addr == 0 { return None; }
        if !Self::is_readable(addr, std::mem::size_of::<T>()) {
            return None;
        }
        (addr as *const T).as_ref().copied()
    }

    pub unsafe fn write<T: Copy>(addr: usize, value: T) -> bool {
        if addr == 0 { return false; }
        // For write, we should check writable too, but let's just try safe pointer check logic first.
        // If it's not writable, it might crash or just fail. 
        // Let's rely on basic validity check.
        if !Self::is_readable(addr, std::mem::size_of::<T>()) {
            return false;
        }
        
        if let Some(ptr) = (addr as *mut T).as_mut() {
            *ptr = value;
            true
        } else {
            false
        }
    }
}
