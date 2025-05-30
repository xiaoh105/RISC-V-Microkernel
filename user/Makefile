TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := ~/.cargo/bin/rust-objdump --arch-name=riscv64
OBJCOPY := ~/.cargo/bin/rust-objcopy --binary-architecture=riscv64

elf:
	@cargo build --release

$(TARGET_DIR)/%.bin: elf
	$(OBJCOPY) $(TARGET_DIR)/$* --strip-all -O binary $@

build: $(BINS)