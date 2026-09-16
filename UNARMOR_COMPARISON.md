# Unarmor optimization and reference benchmark

`messages::unarmor` now uses a 256-entry lookup table and four-character
to three-byte packing. The public API, forward byte order, output length,
fill-bit treatment and error reporting are preserved. Production code contains
only the optimized implementation, including its own invalid-character handling.

The complete original function is frozen in `tests/support/unarmor_reference.rs`.
Unit tests include it only under `cfg(test)`; the benchmark example includes the
same file. It is not compiled into normal library builds. Only the imports around
the reference function have changed; its body is unchanged from upstream
`2b190f587dab55de17cc1ebbde3cd2599cf02f69`. Keep it unchanged as an independent oracle.

The method is based on Holm and Mellegård, *Fast Decoding of Automatic
Identification Systems (AIS) Data*, COMPIT 2018:
<https://www.researchgate.net/publication/325301032>.
This experiment does not port the C decoder's reversed storage or wide field reads.

## Reproduce

```sh
cargo test
cargo test --no-default-features
cargo test --no-default-features --features alloc
cargo run --release --example compare_unarmor
sh scripts/unarmor-assembly.sh
```

The benchmark uses runtime function pointers and `black_box`, includes output
allocation and destruction, and reports medians of nine samples after warmup.
Each sample performs 100,000 calls; set `ITERATIONS` to change this.
Order alternates between implementations. Inputs are deterministic synthetic
valid payloads spanning both armor ranges, not a representative traffic corpus.
The 511-character case is a stress case. These are unarmor timings, not full
message-decoding timings. CPU scheduling and clock variation affect results.

## Local results (2026-09-16)

Rust 1.97.1, aarch64, default Cargo release profile, default `std` feature:

| Input characters | Original ns/call | Optimized ns/call | Speedup |
| --- | ---: | ---: | ---: |
| 28 | 47.33 | 19.05 | 2.48x |
| 71 | 104.86 | 30.87 | 3.40x |
| 168 | 245.69 | 62.50 | 3.93x |
| 511 | 688.43 | 162.36 | 4.24x |

These measurements were rerun after moving the reference into test support.
The compiled original function contains 168 static instructions; the optimized
function contains 230. These include setup, tail, fill, error and unwind paths,
but exclude the bodies of called functions and the 256-byte lookup table.
Code size is not the number of instructions executed per payload.

Inspecting the aarch64 hot loops gives 29 instructions per four characters for
the optimized version, versus 102–118 for four iterations of the original,
depending on armor range. These are counts derived from the successful loop
paths, including loop control and checks, excluding setup, tail and fill handling;
they are not hardware-counter measurements. Compiler/target changes may change
all these counts. The assembly script prints both complete functions for review.

Hardware instruction counting was unavailable in this environment: `perf stat`
reported no supported events and `perf_event_paranoid=4`, also outside the sandbox.
On a machine with counters available, use:

```sh
cargo build --release --example compare_unarmor
perf stat -r 5 -e instructions,cycles target/release/examples/compare_unarmor original
perf stat -r 5 -e instructions,cycles target/release/examples/compare_unarmor optimized
```

Those counters cover the whole process (including benchmark setup, allocator and
warmup), not just unarmor. Both modes use the same iteration counts and inputs.
The program's initial correctness comparisons also invoke both functions once
per payload size, irrespective of mode.
