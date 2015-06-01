#![feature(env)]

extern crate install;
use std::os;



use std::env;

#[cfg(not(test))]
fn main() {
    //println!("Hello, world!")
    let args: Vec<_> = env::args().collect();
    install::uumain(args);
}
