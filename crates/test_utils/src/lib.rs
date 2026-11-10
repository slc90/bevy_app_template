#[cfg(not(all(target_os = "windows", target_pointer_width = "64")))]
compile_error!("This template supports Windows x64 only");

pub mod benchmark;
pub mod scene;
