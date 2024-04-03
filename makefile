TOP_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
OUT_DIR=$(TOP_DIR)out

# configuration
ARCH=x86_64
# ARCH=i686
CONFIG=debug
LOADER=multiboot2
QEMU_EXE=qemu-system-x86_64
# QEMU_EXE=qemu-system-i386

# qemu options
MEMORY=4G
CORES=4

TARGET=$(ARCH)-yeetos

LOADER_DIR=$(TOP_DIR)/loaders/$(LOADER)
LOADER_BIN=$(LOADER_DIR)/target/$(TARGET)/$(CONFIG)/loader

KERNEL_DIR=$(TOP_DIR)kernel
KERNEL_BIN=$(KERNEL_DIR)/target/$(TARGET)/$(CONFIG)/kernel

INITRD=$(OUT_DIR)/initrd
ISO=$(OUT_DIR)/yeetos.iso

all: $(ISO)

$(KERNEL_BIN): FORCE
	@cd $(KERNEL_DIR) && cargo build --target triplets/$(TARGET).json

$(LOADER_BIN): FORCE
	@cd $(LOADER_DIR) && cargo build --target triplets/$(TARGET).json

$(INITRD): $(KERNEL_BIN)
	@mkdir -p $(OUT_DIR)
	@$(TOP_DIR)/scripts/mkinitrd.sh -o $(INITRD) -k $(KERNEL_BIN)

$(ISO): $(INITRD) $(LOADER_BIN)
	@mkdir -p $(OUT_DIR)
	@$(TOP_DIR)/scripts/mkiso.sh -o $(ISO) -l $(LOADER_BIN) -i $(INITRD)

clean:
	@ rm -f $(INITRD) $(ISO)

clean-all: clean
	@cd $(KERNEL_DIR) && cargo clean
	@cd $(LOADER_DIR) && cargo clean

dump-kernel:
	@objdump -x $(KERNEL_BIN)

disassemble-kernel:
	@objdump -d --demangle=rust -M intel --disassembler-color=on $(KERNEL_BIN)

dump-loader:
	@objdump -x $(LOADER_BIN)

disassemble-loader:
	@objdump -d --demangle=rust -M intel --disassembler-color=on $(LOADER_BIN)


qemu: $(ISO)
	@$(QEMU_EXE) -smp cpus=$(CORES) --accel kvm -m $(MEMORY) -cdrom $(ISO) -serial stdio 

qemu-no-kvm: $(ISO)
	@$(QEMU_EXE) -smp cpus=$(CORES) -m $(MEMORY) -cdrom $(ISO) -serial stdio 


qemu-debug: $(ISO)
	@$(QEMU_EXE) -S -gdb tcp::9000 -smp cpus=$(CORES) -m $(MEMORY)  -cdrom $(ISO) -serial stdio 

.PHONY: qemu qemu-no-kvm qemu-debug clean clean-all  dump-kernel

# empty targe to force rebuild
FORCE:
