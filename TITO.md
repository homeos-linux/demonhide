# Tito Configuration for DemonHide

This document explains the Tito packaging setup for building Fedora RPM packages.

## Files Added

- `.tito/tito.props` - Tito configuration
- `demonhide.spec` - RPM spec file
- `rel-eng/packages/.package_database` - Package database
- `Makefile` - Build automation
- `.github/workflows/rpm.yml` - GitHub Actions for RPM building

## Quick Start

1. Install Tito and dependencies:
   ```bash
   sudo dnf install tito rpm-build rpmdevtools
   make dev-deps
   ```

2. Build RPM packages:
   ```bash
   make tito-build
   ```

## Workflow

1. **Development**: Make changes, commit to git
2. **Tag Release**: `make tito-tag` (creates git tag and updates spec)
3. **Build RPM**: `make tito-build` (creates RPM packages)
4. **Test**: Install and test the RPM package

## Commands

- `tito tag` - Tag a new release (increments version)
- `tito build --rpm` - Build binary RPM
- `tito build --srpm` - Build source RPM only
- `tito build --test --rpm` - Test build without tagging

## Files Generated

- `/tmp/tito/demonhide-*.src.rpm` - Source RPM
- `~/rpmbuild/RPMS/x86_64/demonhide-*.rpm` - Binary RPM
- Updated changelog in spec file

## Notes

- Tito automatically updates the spec file changelog
- Version is managed by git tags
- Requires clean git working directory for tagging
- GitHub Actions automatically builds RPMs on tag push