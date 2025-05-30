build:
	cargo build

build_release:
	cargo build --release

build_user:
	cd ../user && make build

remove_metadata: build_release
	~/.cargo/bin/rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os \
	  -O binary target/riscv64gc-unknown-none-elf/release/os.bin
	  

build_all: remove_metadata build_user

qemu_start: build_all
	qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios none \
            -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80000000

qemu_start_gdb: build_all
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
