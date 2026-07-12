binary=rsafe
target=./target/release/$(binary)

.PHONY: clean build all

all: build move
	@echo "Done :)"
	@echo "rsafe -> should work now :)"

build:
	cargo build --release
	strip $(target)

mobile:
	rustup target add arm-linux-androideabi
	cargo build --target=arm-linux-androideabi

move:
	@echo "Moving the binary to /usr/bin"
	sudo mv $(target) /usr/bin/$(binary)

clean:
	cargo clean
