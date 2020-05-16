#![cfg(test)]
#![feature(test)]

extern crate alloc;
extern crate test;

use alloc::vec::Vec;
use hop_engine::{
    command::{CommandId, Request},
    Hop,
};
use test::Bencher;

#[bench]
fn bench_increment(b: &mut Bencher) {
    let hop = Hop::new();
    let args = [b"foo".to_vec()].to_vec();
    let req = Request::new(CommandId::Increment, Some(args));
    let mut resp = Vec::new();

    b.iter(|| {
        hop.dispatch(&req, &mut resp).unwrap();
    });
}
