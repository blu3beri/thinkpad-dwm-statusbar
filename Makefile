build:
	cargo build --release

install:
	mv target/release/status_bar /usr/local/bin/status_bar
