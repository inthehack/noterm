fn main() {
    let mut output = String::new();
    let count = 10;
    noterm::print!(output, "hello {} {}", count, "world");
}
