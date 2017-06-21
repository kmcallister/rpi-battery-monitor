extern crate gcc;

fn main() {
    gcc::compile_library("libgpio.a", &["src/cbits.c"]);
}
