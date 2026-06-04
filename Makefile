
all: debug


debug:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .
