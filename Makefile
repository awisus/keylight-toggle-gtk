all: build

build:
	cargo build --release

run:
	cargo run

install: build
	cp target/release/keylight-toggle-gtk $$HOME/.local/bin/keylight-toggle-gtk
	cp assets/keylight-toggle.png $$HOME/.local/share/icons/keylight-toggle.png

clean:
	rm -rf target
