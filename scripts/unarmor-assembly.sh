#!/bin/sh
# Run from the repository root. Counts static instructions, not executed events.
set -eu
cargo build --release --example compare_unarmor
objdump -d -C --no-show-raw-insn target/release/examples/compare_unarmor |
awk '
/<(ais::messages|compare_unarmor::unarmor_reference)::unarmor>:$/ { active=1; count=0; print; next }
active && /^$/ { print "Static instruction count: " count; print ""; active=0 }
active && /^[[:space:]]*[0-9a-f]+:/ { print; count++ }
'
