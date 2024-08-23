set shell := ["nu", "-c"]
rustflags := if os() == "windows" { "-L ~/.bin" } else { "/usr/bin" }

test:
	cargo test

lint:
	@cargo fmt

run *flags="":
	@RUSTFLAGS="{{rustflags}}" cargo run -q -- {{flags}}
