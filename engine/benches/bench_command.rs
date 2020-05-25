#![cfg(test)]
#![feature(test)]

extern crate alloc;
extern crate test;

use alloc::vec::Vec;
use hop_engine::{
    command::{request::RequestBuilder, CommandId},
    Hop,
};
use test::Bencher;

#[bench]
fn bench_increment(b: &mut Bencher) {
    let hop = Hop::new();
    let mut builder = RequestBuilder::new(CommandId::Increment);
    builder.bytes(b"foo".as_ref()).unwrap();
    let req = builder.into_request();
    let mut resp = Vec::new();

    b.iter(|| {
        hop.dispatch(&req, &mut resp).unwrap();
    });
}
