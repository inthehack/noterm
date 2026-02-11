//! Macros.

#[cfg(test)]
mod tests;

#[macro_export]
#[doc(hidden)]
macro_rules! csi {
    ($($item:expr),*) => {
        concat!("\x1b[", $($item),*)
    }
}

#[macro_export]
macro_rules! print {
    ($output:expr, $fmt:expr $(, $($args:expr),*)? $(,)?) => {{
        use $crate::Executable;
        use $crate::io::blocking::Write;
        use $crate::style::Print;
        $output.execute(Print(format_args!($fmt $(, $($args),*)?))).unwrap();
    }};
}

#[macro_export]
macro_rules! println {
    ($output:expr, $fmt:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::print!($output, concat!($fmt, "\n") $(, $($args),*)?);
    }};
}
