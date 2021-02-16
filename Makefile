

all: test 


test:
	cargo test --verbose
	cargo test --verbose --package fsa-net-parser
	cargo test --verbose --package item-location-derive

build:
	cargo build --verbose
	cargo build --verbose --package fsa-net-parser
	cargo build --verbose --package item-location-derive

release: test
	cargo build --verbose --release
