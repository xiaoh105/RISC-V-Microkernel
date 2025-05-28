build:
	cargo build

build_release:
	cargo build --release

remove_metadata: build_release
	~/.cargo/bin/rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os \
	  -O binary target/riscv64gc-unknown-none-elf/release/os.bin

qemu_start: remove_metadata
	qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios none \
            -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80000000

qemu_start_gdb: remove_metadata
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios none \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80000000 \
        -s -S

qemu_attach_gdb:
	riscv64-elf-gdb \
        -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
        -ex 'set arch riscv:rv64' \
        -ex 'target remote localhost:1234'
