# COPR Makefile for DemonHide
# This file is used by COPR's make_srpm build method

# Variables that COPR provides
# outdir - output directory for SRPM
# spec - path to the project directory

PACKAGE_NAME = demonhide
SPEC_FILE = $(PACKAGE_NAME).spec
VERSION = $(shell grep '^Version:' $(SPEC_FILE) | awk '{print $$2}')
RELEASE = $(shell grep '^Release:' $(SPEC_FILE) | awk '{print $$2}' | cut -d'%' -f1)

.PHONY: srpm

srpm:
	@echo "=== COPR SRPM Build for $(PACKAGE_NAME) ==="
	@echo "Building $(PACKAGE_NAME)-$(VERSION)-$(RELEASE)"
	@echo "Output directory: $(outdir)"
	@echo "Spec directory: $(spec)"
	
	# Set up RPM build tree
	mkdir -p ~/rpmbuild/{SOURCES,SPECS,BUILD,RPMS,SRPMS}
	
	# Create source tarball
	@echo "Creating source tarball..."
	@if command -v git >/dev/null 2>&1; then \
		echo "Using git archive to create tarball..."; \
		git archive --format=tar.gz --prefix="$(PACKAGE_NAME)-$(VERSION)/" HEAD > ~/rpmbuild/SOURCES/$(PACKAGE_NAME)-$(VERSION).tar.gz; \
	else \
		echo "Git not available, creating tarball from current directory..."; \
		tar --exclude='.git' --exclude='target' --exclude='*.rpm' --exclude='*.src.rpm' \
		    --exclude='rpmbuild' --exclude='build' --transform 's,^,$(PACKAGE_NAME)-$(VERSION)/,' \
		    -czf ~/rpmbuild/SOURCES/$(PACKAGE_NAME)-$(VERSION).tar.gz *; \
	fi
	
	# Build SRPM
	@echo "Building SRPM..."
	rpmbuild -bs $(SPEC_FILE) --define "_topdir $$HOME/rpmbuild"
	
	# Copy SRPM to output directory
	@echo "Copying SRPM to output directory..."
	cp ~/rpmbuild/SRPMS/$(PACKAGE_NAME)-$(VERSION)-$(RELEASE)*.src.rpm $(outdir)/
	
	@echo "=== SRPM Build Complete ==="
	@ls -la $(outdir)/$(PACKAGE_NAME)-$(VERSION)-$(RELEASE)*.src.rpm