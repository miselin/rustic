
# Variables defined before the inclusion of config.mk can be overridden in
# config.mk in order to adjust how the build runs.

RUST_ROOT :=
LLVM_ROOT :=
GCC_PREFIX := /usr/bin/

MKISOFS := mkisofs

# Path to source files (if not '.')
SRCDIR := $(shell pwd)

# Directory in which built object files will be placed (if not $(SRCDIR)/build).
BUILDDIR := $(SRCDIR)/build

# Set to the desired linker emulation for the kernel binary.
MACHINE := elf_i386

# Set to the desired target for Rust and LLVM to use.
TARGET := i386-unknown-linux-gnu

# Set to 'true' if GNU as should be used (under $(GCC_PREFIX)) rather than clang
# to assemble assembly code.
USE_GCC_AS := false

# Set to 'gcc -E' if that makes more sense for the target.
PREPROCESSOR := cpp

# Set to 'true' to build required Rust libraries as part of the Rustic build,
# rather than using the ones provided by the system. Handy for cross-compiling
# from OSX!
BUILD_RUST_LIBS := false

# Set to the path to a checkout of https://github.com/mozilla/rust (or your own
# fork) - needed libraries will be built as part of the build process if
# $(BUILD_RUST_LIBS) == 'true'.
RUST_CHECKOUT :=

# Preprocessor definitions to pass in when compiling assembly.
ASDEFS :=

# Configurations to apply to the Rust compiler. Default set is enough for x86.
RUSTIC_CONFIGS := --cfg plat_pc --cfg arch_i386

# Path to rust file for the application crate to build.
APPLICATION_PATH := $(SRCDIR)/src/application/example.rs

# Override this to redefine the location of the config file.
CONFIG ?= $(SRCDIR)/config.mk

-include $(CONFIG)

LIBGCC := $(shell $(GCC_PREFIX)gcc -print-file-name=libgcc.a)

ifeq ($(BUILD_RUST_LIBS), true)
LIBPATH := $(BUILDDIR)/libs
RUST_LIBS := $(BUILDDIR)/libs/libmorestack.a $(BUILDDIR)/libs/libcompiler-rt.a $(BUILDDIR)/libs/libcore.rlib $(BUILDDIR)/libs/librlibc.rlib $(BUILDDIR)/libs/liblibc.rlib $(BUILDDIR)/libs/liballoc.rlib $(BUILDDIR)/libs/libunicode.rlib $(BUILDDIR)/libs/libcollections.rlib $(BUILDDIR)/libs/librand.rlib

# RUST_LIBS_STD lists any Rust libraries that depend on libstd, and therefore
# must be built after the drop-in Rustic libstd is built.
# Wishlist for RUST_LIBS_STD:
# * arena
# * debug
# * flate
# * fmt_macros
# * fourcc
# * green (depends on: std::os, std::rt, std::sync - can we implement these?)
# * hexfloat (depends on syntax)
# * num (core::num does not provide enough support)
# * rustrt (depends on rustrt_native, which we can provide)
# * sync (depends on rustrt)
# * syntax (depends on fmt_macros)
RUST_LIBS_STD :=
else
LIBPATH := $(RUST_ROOT)/lib/rustlib/$(TARGET)/lib
RUST_LIBS :=
RUST_LIBS_STD :=
endif

CLANG := $(LLVM_ROOT)/bin/clang

RC := $(RUST_ROOT)/bin/rustc
RCFLAGS := -O -L $(LIBPATH) -L $(BUILDDIR) --target $(TARGET) -Z no-landing-pads $(RUSTIC_CONFIGS)

LD := $(GCC_PREFIX)ld
LDFLAGS := -m $(MACHINE) -flto --gc-sections -nostdlib -static -Tsrc/linker.ld
LIBS := $(LIBGCC) -L$(LIBPATH) -L$(BUILDDIR) -lmorestack

AR := $(LLVM_ROOT)/bin/llvm-ar
CPP := $(GCC_PREFIX)$(PREPROCESSOR)

ifeq ($(USE_GCC_AS), true)
AS := $(GCC_PREFIX)gcc
ASFLAGS :=
else
AS := $(LLVM_ROOT)/bin/clang
ASFLAGS := -O3 -target $(TARGET)
endif

OBJDIR := $(BUILDDIR)/obj
LIBDIR := $(BUILDDIR)/libs

IMAGESDIR := images

LIBRUSTIC := $(BUILDDIR)/librustic.rlib
LIBRUSTIC_SRCS := $(SRCDIR)/src/rustic.rs

LIBSTD := $(BUILDDIR)/libstd.rlib
LIBSTD_SRCS := $(SRCDIR)/src/std/lib.rs

SRCS := $(APPLICATION_PATH)
ASMSRCS := $(SRCDIR)/src/start.s

OBJS := $(patsubst %.s,$(OBJDIR)/%.s.o,$(ASMSRCS)) $(patsubst %.rs,$(OBJDIR)/%.built.o,$(SRCS))

KERNEL := $(BUILDDIR)/kernel
ISO := $(BUILDDIR)/rustic.iso

LD_LIBRARY_PATH := $(RUST_ROOT)/lib
DYLD_LIBRARY_PATH := $(RUST_ROOT)/lib

.EXPORT_ALL_VARIABLES:
.PHONY: clean all

all: $(RUST_LIBS) $(LIBSTD) $(RUST_LIBS_STD) $(LIBRUSTIC) $(KERNEL) $(ISO)

nolibs: $(LIBRUSTIC) $(KERNEL) $(ISO)

onlylibs: $(RUST_LIBS) $(LIBSTD) $(RUST_LIBS_STD)

onlystdlibs: $(RUST_LIBS_STD)

$(ISO): $(KERNEL)
	@echo "[ISO ]" $@
	@cp $(IMAGESDIR)/grub/stage2_eltorito-x86 $(SRCDIR)/stage2_eltorito
	@$(MKISOFS) -D -joliet -quiet -input-charset iso8859-1 -R \
		-b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4 \
		-boot-info-table -o $@ -V 'RUSTIC' -graft-points \
	    boot/grub/stage2_eltorito=./stage2_eltorito \
	    boot/grub/menu.lst=$(IMAGESDIR)/grub/menu.lst \
	    boot/kernel=$(KERNEL)
	@rm -f $(SRCDIR)/stage2_eltorito

$(KERNEL): $(OBJS)
	@echo "[LINK]" $@
	@$(LD) $(LDFLAGS) -o $@ --whole-archive $^ --no-whole-archive $(LIBS)

$(LIBRUSTIC): $(LIBRUSTIC_SRCS)
	@-mkdir -p `dirname $@`
	@echo "[RC  ]" $@
	@-rm -f $@
	@$(RC) --crate-type=lib $(RCFLAGS) -o $@ $^

$(LIBSTD): $(LIBSTD_SRCS)
	@-mkdir -p `dirname $@`
	@echo "[RC  ]" $@
	@-rm -f $@
	@$(RC) --crate-type=lib $(RCFLAGS) -o $@ $^

$(OBJDIR)/%.built.o: %.rs
	@-mkdir -p `dirname $@`
	@echo "[RC  ]" $@
	@-rm -f $@
	@$(RC) --crate-type=staticlib $(RCFLAGS) -o $@ $^
	@$(AR) t $@ | grep .bytecode | xargs -n 1 $(AR) dv $@

$(OBJDIR)/%.s.o: %.s
	@-mkdir -p `dirname $@`
	@echo "[AS  ]" $@
	@$(CPP) $(ASDEFS) $^ -o $@.S
	@$(AS) $(ASFLAGS) -o $@ -c $@.S
	@rm -f $@.S

$(LIBDIR)/libmorestack.a: $(RUST_CHECKOUT)/src/rt/arch/i386/morestack.S
	@-mkdir -p `dirname $@`
	@echo "[AS  ]" $@
	@$(AS) $(ASFLAGS) -D__linux__ -o $@.o -c $^
	@$(AR) cru $@ $@.o
	@rm -f $@.o

# Because libcompiler-rt is essentially LLVM's replacement for libgcc, we can
# cheat here for cross-compiling and use the cross-compiler's libgcc.a :-)
$(LIBDIR)/libcompiler-rt.a: $(RUST_CHECKOUT)/src/compiler-rt/Makefile
	@echo "[LN  ]" $@
	@ln -sf $(LIBGCC) $@

$(LIBDIR)/lib%.rlib: $(RUST_CHECKOUT)/src/lib%/lib.rs
	@-mkdir -p `dirname $@`
	@echo "[RC  ]" $@
	@$(RC) --crate-type=lib $(RCFLAGS) -o $@ $^

clean:
	-rm -f $(KERNEL)
	-rm -f $(OBJS)
	-rm -rf $(RUST_LIBS)
