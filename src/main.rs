extern crate install;
use std::os;

#[cfg(not(test))]
fn main() {
    //println!("Hello, world!")
    install::uumain(os::args());
}
