
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

# Architecture to use when building libmorestack
MORESTACK_ARCH := i386

# Set to the desired linker emulation for the kernel binary.
MACHINE := elf_i386

# Set to the desired target for Rust and LLVM to use.
TARGET := i386-unknown-linux-gnu

# Set to 'true' if GNU as should be used (under $(GCC_PREFIX)) rather than clang
# to assemble assembly code.
USE_GCC_AS := false

# Set to 'gcc -E' if that makes more sense for the target.
PREPROCESSOR := cpp

# Set to the path to a checkout of https://github.com/mozilla/rust (or your own
# fork) - needed libraries will be built as part of the build process if
# $(BUILD_RUST_LIBS) == 'true'.
# Leave default to have the repository pulled as part of the build process.
RUST_CHECKOUT := $(BUILDDIR)/rust

# Preprocessor definitions to pass in when compiling assembly.
ASDEFS :=

# Preprocessor definitions for the C compiler. Default is good enough for x86.
CPPDEFS := -DPLAT_PC -DARCH_I386

# Configurations to apply to the Rust compiler. Default set is enough for x86.
RUSTIC_CONFIGS := --cfg plat_pc --cfg arch_i386

# Path to rust file for the application crate to build.
APPLICATION_PATH := $(SRCDIR)/src/example/example.rs

# Path to linker script to use for the build. Default is good enough for x86.
LINKER_SCRIPT := $(SRCDIR)/src/linker.ld

# Override this to redefine the location of the config file.
CONFIG ?= $(SRCDIR)/config.mk

-include $(CONFIG)

RUST_REPO := "https://github.com/mozilla/rust"

LIBGCC := $(shell $(GCC_PREFIX)gcc -print-file-name=libgcc.a)

LIBPATH := $(BUILDDIR)/libs
RUST_LIBS := $(BUILDDIR)/libs/libmorestack.a $(BUILDDIR)/libs/libcompiler-rt.a $(BUILDDIR)/libs/libcore.rlib $(BUILDDIR)/libs/librlibc.rlib $(BUILDDIR)/libs/liblibc.rlib $(BUILDDIR)/libs/liballoc.rlib $(BUILDDIR)/libs/libunicode.rlib $(BUILDDIR)/libs/libcollections.rlib $(BUILDDIR)/libs/librand.rlib

# RUST_LIBS_STD lists any Rust libraries that depend on libstd, and therefore
# must be built after the drop-in Rustic libstd is built.
# Wishlist for RUST_LIBS_STD:
# * arena (needs std::rt)
# * debug (needs std::gc, std::io)
# * flate (needs std::c_vec)
# * fmt_macros (needs ::std::fmt::format)
# * fourcc (needs syntax)
# * green (depends on: std::os, std::rt, std::sync - can we implement these?)
# * hexfloat (depends on syntax)
# * num (core::num does not provide enough support)
# * syntax (depends on fmt_macros)
RUST_LIBS_STD := $(BUILDDIR)/libs/liblibc.rlib $(BUILDDIR)/libs/librustrt.rlib $(BUILDDIR)/libs/libsync.rlib

CLANG := $(LLVM_ROOT)/bin/clang

RC := $(RUST_ROOT)/bin/rustc
RCFLAGS := -O -L $(LIBPATH) -L $(BUILDDIR) --target $(TARGET) -Z no-landing-pads $(RUSTIC_CONFIGS)

RUSTDOC := $(RUST_ROOT)/bin/rustdoc
RUSTDOC_FLAGS := $(RUSTIC_CONFIGS)

CC := $(GCC_PREFIX)gcc
CFLAGS := -O3

LD := $(GCC_PREFIX)ld
LDFLAGS := -m $(MACHINE) -flto --gc-sections -nostdlib -static -T$(LINKER_SCRIPT)
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
LIBRUSTIC_SRCS := $(SRCDIR)/src/rustic/rustic.rs

LIBRUSTRT_NATIVE := $(BUILDDIR)/librustrt_native.a
LIBRUSTRT_NATIVE_SRCS := $(SRCDIR)/src/rustrt_native/rustrt.c

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
.PHONY: checkenv bootstrap runtime onlylibs rustic app clean all

################################################################################

all: rust_src checkenv bootstrap runtime onlylibs rustic app

checkenv:
	@[[ -d "$(RUST_CHECKOUT)" ]] || echo "Please set RUST_CHECKOUT to a valid directory."
	@[[ -e "$(APPLICATION_PATH)" ]] || echo "APPLICATION_PATH is set to a file that does not exist."
	@[[ -x "$(CC)" ]] || echo "Please make sure GCC_PREFIX is set correctly (can't execute GCC)."
	@[[ -x "$(LD)" ]] || echo "Please make sure GCC_PREFIX is set correctly (can't execute LD)."
	@[[ -x "$(RC)" ]] || echo "Please make sure RUST_ROOT is set correctly (can't execute the Rust compiler)."
	@[[ -x "$(AR)" ]] || echo "Please make sure LLVM_ROOT is set correctly (can't execute llvm-ar)."

# We need to have a bootstrap step to build a bootstrap runtime. In the
# bootstrap runtime, libstd does not pull in the 'sync' or 'rustrt' crates,
# both of which depend on libstd.
bootstrap:
	@[[ -e $(LIBSTD) ]] || (echo "Bootstrapping Rustic's Rust runtime..."; \
	make -B -C . runtime RUSTIC_CONFIGS="$(RUSTIC_CONFIGS) --cfg bootstrap" CONFIG=$(CONFIG) -j1; \
	echo "Bootstrap complete!")

runtime: $(RUST_LIBS) $(LIBRUSTRT_NATIVE) $(RUST_LIBS_STD) $(LIBSTD)

rustic: $(LIBRUSTIC)

app: $(KERNEL) $(ISO)

ifeq ($(RUST_CHECKOUT), $(BUILDDIR)/rust)
rust_src:
	@[[ -e $(RUST_CHECKOUT) ]] && (cd $(RUST_CHECKOUT) && git pull) || (cd `dirname $(RUST_CHECKOUT)` && git clone $(RUST_REPO))
else
rust_src:
endif

################################################################################

doc:
	$(RUSTDOC) $(RUSTDOC_FLAGS) $(LIBRUSTIC_SRCS)

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

$(LIBRUSTRT_NATIVE): $(LIBRUSTRT_NATIVE_SRCS)
	@-mkdir -p `dirname $@`
	@echo "[CC  ]" $@
	@$(CC) $(CFLAGS) -o $@.o -c $^
	@$(AR) cru $@ $@.o
	@rm -f $@.o

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

$(LIBDIR)/libmorestack.a: $(RUST_CHECKOUT)/src/rt/arch/$(MORESTACK_ARCH)/morestack.S
	@-mkdir -p `dirname $@`
	@echo "[AS  ]" $@
	@$(AS) $(ASFLAGS) -D__linux__ -o $@.o -c $^
	@$(AR) cru $@ $@.o
	@rm -f $@.o

# Because libcompiler-rt is essentially LLVM's replacement for libgcc, we can
# cheat here for cross-compiling and use the cross-compiler's libgcc.a :-)
$(LIBDIR)/libcompiler-rt.a:
	@echo "[LN  ]" $@
	@ln -sf $(LIBGCC) $@

$(LIBDIR)/lib%.rlib: $(RUST_CHECKOUT)/src/lib%/lib.rs
	@-mkdir -p `dirname $@`
	@echo "[RC  ]" $@
	@$(RC) --crate-type=lib $(RCFLAGS) -o $@ $^

clean:
	-rm -f $(RUST_LIBS) $(LIBRUSTRT_NATIVE) $(RUST_LIBS_STD) $(LIBSTD) $(LIBRUSTIC) $(KERNEL) $(ISO)
