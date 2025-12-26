//! Macros.

#[macro_export]
macro_rules! csi {
    ($($item:expr),+ $(,)?) => {
        concat!("\x1b[", $($item),+)
    }
}

#[macro_export]
macro_rules! write {
    ($writer:ident, $format:expr) => {
        $writer.write_all($format.as_bytes())
    };

    ($writer:ident, $format:expr, $($item:expr),+) => {
        $writer.write_all(
            format_args!($format, $($item),+)
                .as_str()
                .expect("should format arguments")
                .as_bytes()
        )
    };
}
