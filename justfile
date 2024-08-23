set shell := ["nu", "-c"]
rustflags := if os() == "windows" { "-L ~/.bin" } else { "/usr/bin" }

check: lint test

test:
	@ CORUND_MODE=test cargo test

lint:
	@ cargo fmt

run *flags="":
	@ CORUND_MODE=dev RUSTFLAGS="{{rustflags}}" cargo run -q -- {{flags}}
