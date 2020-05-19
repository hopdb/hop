#![cfg(test)]
#![feature(test)]

extern crate test;

use core::iter;
use hop_engine::command::{
    request::Context as RequestContext,
    response::{Context as ResponseContext, Instruction as ResponseInstruction, Response},
};
use test::Bencher;

#[bench]
fn bench_simple_increment(b: &mut Bencher) {
    let mut ctx = RequestContext::new();
    let input = test::black_box([0].to_vec());

    b.iter(|| {
        ctx.feed(&input).unwrap();
    });
}

#[bench]
fn bench_response_list(b: &mut Bencher) {
    let mut list: Vec<Vec<u8>> = Vec::new();
    for _ in 0..15 {
        list.push(iter::repeat(b'a').take(10).collect::<Vec<u8>>());
    }

    let response = Response::from(list);
    let bytes = response.as_bytes();
    let mut ctx = ResponseContext::new();

    b.iter(|| {
        assert!(matches!(
            ctx.feed(&bytes),
            Ok(ResponseInstruction::Concluded(_))
        ));
    });
}
