extern crate cc;

fn main() {
    cc::Build::new().file("src/x.c").compile("x");
}
