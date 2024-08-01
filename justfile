set shell := ["nu", "-c"]

run *flags="":
	@cargo run -q -- {{flags}}

lint:
	@cargo fmt
