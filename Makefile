all: build

build:
	cargo build --release

run:
	cargo run

install: build
	cp target/release/keylight-toggle $$HOME/.local/bin/keylight-toggle
	cp assets/keylight-toggle.png $$HOME/.local/share/icons/keylight-toggle.png

clean:
	cargo clean
