TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
USER_FILE := $(shell find . -path "./target" -prune -o -type f)
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := ~/.cargo/bin/rust-objdump --arch-name=riscv64
OBJCOPY := ~/.cargo/bin/rust-objcopy --binary-architecture=riscv64

elf: $(USER_FILE)
	@cargo build --release

$(TARGET_DIR)/%.bin: $(TARGET_DIR)/%
	$(OBJCOPY) $(TARGET_DIR)/$* --strip-all -O binary $@

build: elf $(BINS)