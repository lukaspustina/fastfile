#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::read_advise;
#[cfg(target_os = "macos")]
pub use macos::read_ahead;
#[cfg(target_os = "macos")]
pub use macos::get_page_cache_info;

// TODO: PAGE_SIZE should be either determined during run or even better during compile time
// 
// fn get_sys_page_size() -> libc::c_long {
//     unsafe {
//         libc::sysconf(libc::_SC_PAGESIZE)
//     }
// }
pub const PAGE_SIZE: usize = 4096;

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

