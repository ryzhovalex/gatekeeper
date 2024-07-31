set shell := ["nu", "-c"]

run:
	@cargo run -q

lint:
	@cargo fmt
