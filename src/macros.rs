//! Macros.

#[macro_export]
macro_rules! csi {
    ($($item:expr),*) => {
        concat!("\x1b[", $($item),*)
    }
}
