#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::get_page_cache_info;
#[cfg(target_os = "macos")]
pub use macos::read_advise;
#[cfg(target_os = "macos")]
pub use macos::read_ahead;

// pub const PAGE_SIZE: usize = ???;
include!(concat!(env!("OUT_DIR"), "/os_consts.rs"));

pub struct PageCacheInfo {
    total: usize,
    cached: usize,
}

impl PageCacheInfo {
    pub fn total(&self) -> usize {
        self.total
    }

    pub fn cached(&self) -> usize {
        self.cached
    }

    pub fn ratio(&self) -> f32 {
        self.cached as f32 / self.total as f32
    }
}
