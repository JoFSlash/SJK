//! The graphics card's video memory, for SJK's small-card settings
//! (`docs/rendering.md`, "GPU memory"), behind a safe API.
//!
//! wgpu names the adapter but not its memory. On Windows, DXGI says both: the
//! card's own memory (`DedicatedVideoMemory`, 3 GB on a 3 GB card) and the budget
//! Windows gives this process now (`QueryVideoMemoryInfo`, less while other programs
//! hold some). [`high_performance`] reads the adapter DXGI ranks first for high
//! performance, the one wgpu's `PowerPreference::HighPerformance` picks, skipping
//! software adapters. Elsewhere it reads nothing.
//!
//! This crate is allowed `unsafe` (as sjk-mediafoundation is): the calls below are
//! DXGI COM calls.

/// One adapter's video memory, in bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoMemory {
    /// The adapter's name, to check it is the one wgpu chose.
    pub name: String,
    /// The card's own memory; an integrated GPU's is small, as it borrows the system's.
    pub dedicated: u64,
    /// What Windows lets this process use of it now, when DXGI 1.4 says.
    pub budget: Option<u64>,
    /// What this process uses of it now, when DXGI 1.4 says (the VRAM measurements).
    pub usage: Option<u64>,
}

/// The high-performance adapter's memory, `None` without DXGI or an adapter.
#[cfg(windows)]
pub fn high_performance() -> Option<VideoMemory> {
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
        DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO, IDXGIAdapter3,
        IDXGIFactory6,
    };

    // SAFETY: CreateDXGIFactory1 has no preconditions; the factory is a COM pointer
    // released when it drops.
    let factory: IDXGIFactory6 = unsafe { CreateDXGIFactory1() }.ok()?;
    for index in 0.. {
        // SAFETY: any index is valid; past the last adapter the call fails, ending the loop.
        let adapter: IDXGIAdapter3 = unsafe {
            factory.EnumAdapterByGpuPreference(index, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)
        }
        .ok()?;
        // SAFETY: the adapter is a live COM pointer; GetDesc1 only fills the returned value.
        let desc = unsafe { adapter.GetDesc1() }.ok()?;
        if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
        // SAFETY: node 0 exists on every adapter; `info` is a valid place to write to.
        let queried =
            unsafe { adapter.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut info) }
                .ok();
        let budget = queried.map(|()| info.Budget);
        let usage = queried.map(|()| info.CurrentUsage);
        let end = desc.Description.iter().position(|&c| c == 0);
        return Some(VideoMemory {
            name: String::from_utf16_lossy(
                &desc.Description[..end.unwrap_or(desc.Description.len())],
            ),
            dedicated: desc.DedicatedVideoMemory as u64,
            budget,
            usage,
        });
    }
    None
}

/// The high-performance adapter's memory: never known off Windows.
#[cfg(not(windows))]
pub fn high_performance() -> Option<VideoMemory> {
    None
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn a_windows_machine_names_its_card_and_memory() {
        // A machine without a GPU (a CI runner) may have only the software adapter.
        if let Some(memory) = super::high_performance() {
            assert!(!memory.name.is_empty());
            eprintln!("{memory:?}");
        }
    }
}
