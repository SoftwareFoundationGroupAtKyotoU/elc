# The script for executing elcc
elcc: target/release/elcc elcc-memo
	@echo "#! /bin/bash\n" > elcc
	@echo "DYLD_LIBRARY_PATH=$(shell cat elcc-memo)/lib $(shell pwd)/target/release/elcc \$$@" >> elcc
	@chmod 700 elcc
	@echo "Generated elcc"

# Executable of elcc built by cargo
target/release/elcc: **/*.rs Cargo.toml
	cargo build --release

elcc-memo:
	rustc --print sysroot > elcc-memo

clean:
	cargo clean
	rm -f elcc elcc-memo
