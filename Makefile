
# Variables defined before the inclusion of config.mk can be overridden in
# config.mk in order to adjust how the build runs.

RUST_ROOT := $(shell realpath $${HOME}/.cargo)

MKISOFS := mkisofs

# Path to source files (if not '.')
SRCDIR := $(shell pwd)

# Directory in which built object files will be placed (if not $(SRCDIR)/build).
BUILDDIR := $(SRCDIR)/build

# Override default shell (e.g. to avoid using dash)
SHELL := /bin/bash

# Override this to redefine the location of the config file.
CONFIG ?= $(SRCDIR)/config.mk

-include $(CONFIG)

KERNEL := $(BUILDDIR)/rustic-example
ISO := $(BUILDDIR)/rustic.iso

.EXPORT_ALL_VARIABLES:
.PHONY: checkenv bootstrap runtime onlylibs rustic app clean all

################################################################################

all: app

app: $(ISO)

################################################################################

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

clean:
	-rm -f $(ISO)
