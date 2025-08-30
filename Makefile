binary=rsafe
target=./target/release/$(binary)

.PHONY: clean build all

all: build move 
	@echo "Done :)"
	@echo "rsafe -> should work now :)"

build:
	cargo build --release 
	strip $(target)

move: 
	sudo mv $(target) /usr/bin/$(binary)

clean:
	cargo clean
