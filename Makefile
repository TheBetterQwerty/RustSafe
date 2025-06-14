binary=RustSafe
target=./target/debug/$(binary)

.PHONY: clean build 

all: build move 
	@echo "Done :)"

build:
	cargo build --release 
	strip $(target)

move: 
	sudo mv $(target) /usr/bin

clean:
	cargo clean
