
RUST_ROOT :=
LLVM_ROOT :=
GCC_PREFIX := /usr/bin/

MACHINE := elf_i386

MKISOFS := mkisofs

-include ./config.mk

TARGET := i686-intel-linux-elf

CC := $(LLVM_ROOT)/bin/clang
CFLAGS := -O3 -target $(TARGET)

RC := $(RUST_ROOT)/bin/rustc
# TODO (eddyb) replace cfg(libc) in rust-core with cfg(allocator=custom)
RCFLAGS := --opt-level=2 --cfg libc --target $(TARGET)

LD := $(GCC_PREFIX)ld
LDFLAGS := -nostdlib -m $(MACHINE) -Tsrc/linker.ld
LIBS := $(shell $(GCC_PREFIX)gcc -print-file-name=libgcc.a)

AS := $(LLVM_ROOT)/bin/clang
ASFLAGS := -O3 -target $(TARGET)

BUILDDIR := build
OBJDIR := $(BUILDDIR)/obj

IMAGESDIR := images

SRCS := src/main.rs
CSRCS := src/rusty.c
ASMSRCS := src/start.S

OBJS := $(patsubst %.rs,$(OBJDIR)/%.o,$(SRCS)) $(patsubst %.c,$(OBJDIR)/%.c.o,$(CSRCS)) $(patsubst %.S,$(OBJDIR)/%.S.o,$(ASMSRCS))

KERNEL := $(BUILDDIR)/kernel
ISO := $(BUILDDIR)/rustic.iso

.PHONY: clean all

all: $(KERNEL) $(ISO)

$(ISO): $(KERNEL)
	@echo "[ISO ]" $@
	@cp $(IMAGESDIR)/grub/stage2_eltorito-x86 ./stage2_eltorito
	@$(MKISOFS) -D -joliet -quiet -input-charset ascii -R -b stage2_eltorito \
	    -no-emul-boot -boot-load-size 4 -boot-info-table -o $@ -V 'RUSTIC' \
	    ./stage2_eltorito \
	    $(IMAGESDIR)/grub/menu.lst \
	    $(KERNEL)
	@rm -f ./stage2_eltorito

$(KERNEL): $(OBJS)
	@echo "[LINK]" $@
	@$(LD) $(LDFLAGS) -o $@ $^ $(LIBS)

$(OBJDIR)/%.o: %.rs
	@-mkdir -p `dirname $@`
	@echo "[RC  ]" $@
	@$(RC) $(RCFLAGS) -o $@ -c $^

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

