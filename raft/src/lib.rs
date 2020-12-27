#![allow(dead_code)]
#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[allow(unused_imports)]
#[macro_use]
extern crate prost_derive;

pub mod kvraft;
mod proto;
pub mod raft;

fn your_code_here<T>(_: T) -> ! {
    unimplemented!()
}
