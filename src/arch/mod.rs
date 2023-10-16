#![allow(dead_code)]
pub mod linux;

/// Generic `syscalll_name` called by a not-defined table
pub fn syscall_name(rax: u64) -> String {
    rax.to_string()
}
