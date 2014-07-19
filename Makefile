
# Variables defined before the inclusion of config.mk can be overridden in
# config.mk in order to adjust how the build runs.

RUST_ROOT :=
LLVM_ROOT :=
GCC_PREFIX := /usr/bin/

MKISOFS := mkisofs

# Set to the desired linker emulation for the kernel binary.
MACHINE := elf_i386

# Set to the desired BFD target to be used when doing an objcopy, if
# $(RUST_REQUIRES_OBJCOPY) == 'true'.
MACHINE_BFD_TARGET := elf32-i386

# Set to the desired target for Rust and LLVM to use.
TARGET := i686-unknown-linux-gnu

# Set to 'true' if GNU as should be used (under $(GCC_PREFIX)) rather than clang
# to assemble assembly code.
USE_GCC_AS := false

# Set to 'gcc -E' if that makes more sense for the target.
PREPROCESSOR := cpp

# Preprocessor definitions to pass in when compiling assembly.
ASDEFS :=

-include ./config.mk

RC := $(RUST_ROOT)/bin/rustc
RCFLAGS := --opt-level=2 -L $(RUST_ROOT)/lib/rustlib/$(TARGET)/lib --target $(TARGET) -Z no-landing-pads

LD := $(GCC_PREFIX)ld
LDFLAGS := -flto --gc-sections -nostdlib -static -m $(MACHINE) -Tsrc/linker.ld
LIBS := $(shell $(GCC_PREFIX)gcc -print-file-name=libgcc.a)

AR := $(GCC_PREFIX)ar
OBJCOPY := $(GCC_PREFIX)objcopy
CPP := $(GCC_PREFIX)$(PREPROCESSOR)

ifeq ($(USE_GCC_AS), true)
AS := $(GCC_PREFIX)gcc
ASFLAGS :=
else
AS := $(LLVM_ROOT)/bin/clang
ASFLAGS := -O3 -target $(TARGET)
endif

BUILDDIR := build
OBJDIR := $(BUILDDIR)/obj

IMAGESDIR := images

SRCS := src/main.rs
ASMSRCS := src/start.S

OBJS := $(patsubst %.S,$(OBJDIR)/%.s.o,$(ASMSRCS)) $(patsubst %.rs,$(OBJDIR)/%.built.o,$(SRCS))

KERNEL := $(BUILDDIR)/kernel
ISO := $(BUILDDIR)/rustic.iso

LD_LIBRARY_PATH := $(RUST_ROOT)/lib
DYLD_LIBRARY_PATH := $(RUST_ROOT)/lib

.EXPORT_ALL_VARIABLES:
.PHONY: clean all

all: $(KERNEL) $(ISO)

$(ISO): $(KERNEL)
	@echo "[ISO ]" $@
	@cp $(IMAGESDIR)/grub/stage2_eltorito-x86 ./stage2_eltorito
	@$(MKISOFS) -D -joliet -quiet -input-charset iso8859-1 -R -b stage2_eltorito \
	    -no-emul-boot -boot-load-size 4 -boot-info-table -o $@ -V 'RUSTIC' \
	    -graft-points ./stage2_eltorito \
	    /boot/grub/menu.lst=$(IMAGESDIR)/grub/menu.lst \
	    /boot/kernel=$(KERNEL)
	@rm -f ./stage2_eltorito

$(KERNEL): $(OBJS)
	@echo "[LINK]" $@
	@$(LD) $(LDFLAGS) -o $@ $^ $(LIBS)

$(OBJDIR)/%.built.o: %.rs
	@-mkdir -p `dirname $@`
	@echo "[RC  ]" $@
	@$(RC) --crate-type=staticlib $(RCFLAGS) -o $@ $^

$(OBJDIR)/%.s.o: %.s
	@-mkdir -p `dirname $@`
	@echo "[AS  ]" $@
	@$(CPP) $(ASDEFS) $^ -o $@.S
	@$(AS) $(ASFLAGS) -o $@ -c $@.S

clean:
	-rm -f $(KERNEL)
	-rm -f $(OBJS)

