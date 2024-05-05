TOP_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
OUT_DIR=$(TOP_DIR)out

# configuration

# ARCH=i686
ARCH=x86_64
CONFIG=debug
LOADER=multiboot2

IMAGE_SIZE=64M

# qemu options
MEMORY=4G
CORES=4

# end configuration

TARGET:=$(ARCH)-yeetos

UEFI_DIR=$(TOP_DIR)/loaders/uefi
UEFI_BIN=$(UEFI_DIR)/target/$(ARCH)-unknown-uefi/$(CONFIG)/uefi-loader.efi

MULTIBOOT2_DIR=$(TOP_DIR)/loaders/multiboot2
MULTIBOOT2_BIN=$(MULTIBOOT2_DIR)/target/$(TARGET)/$(CONFIG)/multiboot2-loader

KERNEL_DIR:=$(TOP_DIR)kernel
KERNEL_BIN:=$(KERNEL_DIR)/target/$(TARGET)/$(CONFIG)/kernel

KERNEL_CMDLINE:=$(TOP_DIR)kernel_cmdline.cfg
KERNEL_FONT_SRC:=/usr/share/kbd/consolefonts/eurlatgr.psfu.gz

INITRD:=$(OUT_DIR)/initrd
ISO=$(OUT_DIR)/yeetos.iso
UEFI_IMG=$(OUT_DIR)/uefi.img
KERNEL_FONT_OUT:=$(OUT_DIR)/eurlatgr.psfu

QEMU_ARGS:= -smp cpus=$(CORES) -m $(MEMORY)

ifeq ($(CONFIG), debug)
PROFILE := dev
else ifeq ($(CONFIG), release)
PROFILE := release
endif

ifeq ($(ARCH), x86_64)
QEMU_EXE:=qemu-system-x86_64
UEFI_FIRMWARE:=/usr/share/edk2-ovmf/x64/OVMF.fd
else ifeq ($(ARCH), i686)
QEMU_EXE:=qemu-system-i386
UEFI_FIRMWARE:=/usr/share/edk2-ovmf/ia32/OVMF.fd
endif

DEPS:=$(INITRD)

ifeq ($(LOADER), uefi)
DEPS+= $(UEFI_IMG)
QEMU_ARGS+= -bios $(UEFI_FIRMWARE) -drive format=raw,file=$(UEFI_IMG),if=ide -serial stdio 
else ifeq ($(LOADER), multiboot2)
DEPS+= $(ISO)
QEMU_ARGS+= -cdrom $(ISO) -serial stdio 
endif

all: $(DEPS)

$(KERNEL_BIN): FORCE
	@cd $(KERNEL_DIR) && cargo build --profile=$(PROFILE) --target triplets/$(TARGET).json

$(UEFI_BIN): FORCE
	@cd $(UEFI_DIR) && cargo build --profile=$(PROFILE) --target $(ARCH)-unknown-uefi

$(MULTIBOOT2_BIN): FORCE
	@cd $(MULTIBOOT2_DIR) && cargo build --profile=$(PROFILE) --target triplets/$(TARGET).json

$(INITRD): $(KERNEL_BIN) $(KERNEL_CMDLINE) $(KERNEL_FONT_OUT)
	@mkdir -p $(OUT_DIR)
	@$(TOP_DIR)/scripts/mkinitrd.sh -o $(INITRD) -k $(KERNEL_BIN) -c $(KERNEL_CMDLINE) -f $(KERNEL_FONT_OUT)

$(UEFI_IMG): $(INITRD) $(UEFI_BIN)
	@$(TOP_DIR)/scripts/mkimg.sh -o $(UEFI_IMG) -s $(IMAGE_SIZE) -a $(ARCH) -l $(UEFI_BIN) -i $(INITRD)

$(ISO): $(INITRD) $(MULTIBOOT2_BIN)
	@mkdir -p $(OUT_DIR)
	@$(TOP_DIR)/scripts/mkiso.sh -o $(ISO) -l $(MULTIBOOT2_BIN) -i $(INITRD)

$(KERNEL_FONT_OUT): $(KERNEL_FONT_SRC)
	@cp $(KERNEL_FONT_SRC) $(OUT_DIR)/font.gz
	@gunzip $(OUT_DIR)/font.gz
	@mv $(OUT_DIR)/font $(KERNEL_FONT_OUT)


clean:
	@rm -f $(INITRD) $(KERNEL_FONT_OUT) $(ISO) $(UEFI_IMG)

clean-all: clean
	@cd $(KERNEL_DIR) && cargo clean
	@cd $(MULTIBOOT2_DIR) && cargo clean
	@cd $(UEFI_DIR) && cargo clean

dump-kernel:
	@objdump -x $(KERNEL_BIN)

disassemble-kernel:
	@objdump -d --demangle=rust -M intel --disassembler-color=on $(KERNEL_BIN)

dump-multiboot2:
	@objdump -x $(MULTIBOOT2_BIN)

disassemble-multiboot2:
	@objdump -d --demangle=rust -M intel --disassembler-color=on $(MULTIBOOT2_BIN)

dump-uefi:
	@objdump -x $(UEFI_BIN)

disassemble-uefi:
	@objdump -d --demangle=rust -M intel --disassembler-color=on $(UEFI_BIN)

qemu: all
	@$(QEMU_EXE) $(QEMU_ARGS) --accel kvm

qemu-no-kvm: all
	@$(QEMU_EXE) $(QEMU_ARGS)


qemu-debug: all
	@$(QEMU_EXE) $(QEMU_ARGS) -S -gdb tcp::9000

.PHONY: all qemu qemu-no-kvm qemu-debug clean clean-all dump-kernel disassemble-kernel dump-multiboot2 disassemble-multiboot2 dump-uefi disassemble-uefi

# empty targe to force rebuild
FORCE:
