//! Run with: cargo run --release --example compare_unarmor
//! Optional original/optimized argument selects one implementation for perf stat.
use ais::messages::unarmor;
use ais::{errors, lib, sentence};
#[path = "../tests/support/unarmor_reference.rs"]
mod unarmor_reference;
use std::{hint::black_box, time::Instant};

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let iterations: usize = std::env::var("ITERATIONS")
        .ok()
        .map(|s| s.parse().expect("ITERATIONS must be an integer"))
        .unwrap_or(100_000);
    assert!(iterations > 0);
    let functions = [unarmor_reference::unarmor, unarmor];
    println!("payload bytes,original ns/op,optimized ns/op,speedup");
    for len in [28usize, 71, 168, 511] {
        let data: Vec<u8> = (0..len)
            .map(|i| {
                let v = ((i * 37 + 11) % 64) as u8;
                if v < 40 {
                    v + 48
                } else {
                    v + 56
                }
            })
            .collect();
        let fill = if len == 71 { 2 } else { 0 };
        assert_eq!(
            unarmor_reference::unarmor(&data, fill).unwrap(),
            unarmor(&data, fill).unwrap()
        );
        let mut samples = [Vec::new(), Vec::new()];
        // Alternate order to reduce systematic drift; discard the warmup round.
        for round in 0..10 {
            for step in 0..2 {
                let which = (round + step) % 2;
                if (mode == "original" && which != 0) || (mode == "optimized" && which != 1) {
                    continue;
                }
                let function = black_box(functions[which]);
                let start = Instant::now();
                for _ in 0..iterations {
                    black_box(function(black_box(&data), black_box(fill)).unwrap());
                }
                let elapsed = start.elapsed().as_secs_f64() * 1e9 / iterations as f64;
                if round != 0 {
                    samples[which].push(elapsed);
                }
            }
        }
        if mode.is_empty() {
            for sample in &mut samples {
                sample.sort_by(f64::total_cmp);
            }
            let a = samples[0][4];
            let b = samples[1][4];
            println!("{len},{a:.2},{b:.2},{:.2}x", a / b);
        }
    }
}
