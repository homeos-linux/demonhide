# Makefile for DemonHide Tito packaging

.PHONY: help srpm rpm build test-build clean tito-init tito-tag tito-build

# Variables
PACKAGE_NAME = demonhide
SPEC_FILE = $(PACKAGE_NAME).spec

help:
	@echo "DemonHide Tito Packaging Commands:"
	@echo "  help        - Show this help message"
	@echo "  tito-init   - Initialize Tito (run once)"
	@echo "  tito-tag    - Tag a new release"
	@echo "  tito-build  - Build RPM packages"
	@echo "  srpm        - Build source RPM only"
	@echo "  rpm         - Build binary RPM packages"
	@echo "  test-build  - Test build without tagging"
	@echo "  clean       - Clean build artifacts"

tito-init:
	@echo "Initializing Tito..."
	tito init

tito-tag:
	@echo "Tagging new release with Tito..."
	tito tag

tito-build:
	@echo "Building RPM packages with Tito..."
	tito build --rpm

srpm:
	@echo "Building source RPM..."
	tito build --srpm

rpm: srpm
	@echo "Building binary RPM..."
	tito build --rpm

test-build:
	@echo "Testing build without tagging..."
	tito build --test --rpm

clean:
	@echo "Cleaning build artifacts..."
	rm -rf /tmp/tito
	rm -rf build/
	rm -rf *.rpm
	rm -rf *.src.rpm

# Development targets
dev-deps:
	@echo "Installing development dependencies..."
	sudo dnf install -y tito rpm-build rpmdevtools
	rpmdev-setuptree

# Show current package info
info:
	@echo "Package: $(PACKAGE_NAME)"
	@echo "Spec file: $(SPEC_FILE)"
	@echo "Current version:"
	@grep "^Version:" $(SPEC_FILE) || echo "  Version not found in spec"
	@echo "Git tags:"
	@git tag -l | tail -5

# Fedora COPR build (requires copr-cli setup)
copr-build:
	@echo "Building in Fedora COPR..."
	tito build --srpm
	copr-cli build demonhide /tmp/tito/$(PACKAGE_NAME)-*.src.rpm