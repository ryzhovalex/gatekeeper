set shell := ["nu", "-c"]
rustflags := if os() == "windows" { "-L ~/.bin" } else { "/usr/bin" }

check: lint test

test t="":
	@ CORUND_MODE=test cargo test {{t}}

test_exact t="":
	@ CORUND_MODE=test cargo test {{t}} -- --exact

lint:
	@ cargo fmt

run *flags="":
	@ CORUND_MODE=dev RUSTFLAGS="{{rustflags}}" cargo run -- {{flags}}
