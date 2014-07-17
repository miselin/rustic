
RUST_ROOT :=
LLVM_ROOT :=
GCC_PREFIX := /usr/bin/

MACHINE := elf_i386

MKISOFS := mkisofs

-include ./config.mk

TARGET := i686-unknown-linux-gnu

CC := $(LLVM_ROOT)/bin/clang
CFLAGS := -O3 -target $(TARGET)

RC := $(RUST_ROOT)/bin/rustc
RCFLAGS := --opt-level=2 -L $(RUST_ROOT)/lib/rustlib/$(TARGET)/lib --target $(TARGET)

LD := $(GCC_PREFIX)ld
LDFLAGS := -flto --gc-sections -nostdlib -static -m $(MACHINE) -Tsrc/linker.ld
LIBS := $(shell $(GCC_PREFIX)gcc -print-file-name=libgcc.a)

AS := $(LLVM_ROOT)/bin/clang
ASFLAGS := -O3 -target $(TARGET)

BUILDDIR := build
OBJDIR := $(BUILDDIR)/obj

IMAGESDIR := images

SRCS := src/main.rs
CSRCS := src/rusty.c
ASMSRCS := src/start.S

OBJS := $(patsubst %.c,$(OBJDIR)/%.c.o,$(CSRCS)) $(patsubst %.rs,$(OBJDIR)/%.built.o,$(SRCS)) $(patsubst %.S,$(OBJDIR)/%.S.o,$(ASMSRCS))

KERNEL := $(BUILDDIR)/kernel
ISO := $(BUILDDIR)/rustic.iso

LD_LIBRARY_PATH := $(RUST_ROOT)/lib

.EXPORT_ALL_VARIABLES:
.PHONY: clean all

all: $(KERNEL) $(ISO)

$(ISO): $(KERNEL)
	@echo "[ISO ]" $@
	@cp $(IMAGESDIR)/grub/stage2_eltorito-x86 ./stage2_eltorito
	@$(MKISOFS) -D -joliet -quiet -input-charset ascii -R -b stage2_eltorito \
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

$(OBJDIR)/%.c.o: %.c
	@-mkdir -p `dirname $@`
	@echo "[CC  ]" $@
	@$(CC) $(CFLAGS) -o $@ -c $^

$(OBJDIR)/%.S.o: %.S
	@-mkdir -p `dirname $@`
	@echo "[AS  ]" $@
	@$(AS) $(ASFLAGS) -o $@ -c $^

clean:
	-rm -f $(KERNEL)
	-rm -f $(OBJS)

