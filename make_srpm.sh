#!/bin/bash
# Simple SRPM builder for COPR without Tito dependency

set -e

SPEC_FILE="demonhide.spec"
PACKAGE_NAME="demonhide"

# Extract version and release from spec file
VERSION=$(grep '^Version:' "$SPEC_FILE" | awk '{print $2}')
RELEASE=$(grep '^Release:' "$SPEC_FILE" | awk '{print $2}' | cut -d'%' -f1)

echo "Building SRPM for $PACKAGE_NAME-$VERSION-$RELEASE"

# Set up RPM build tree
mkdir -p ~/rpmbuild/{SOURCES,SPECS,BUILD,RPMS,SRPMS}

# Create source tarball from current git tree
echo "Creating source tarball..."
git archive --format=tar.gz --prefix="$PACKAGE_NAME-$VERSION/" HEAD > ~/rpmbuild/SOURCES/"$PACKAGE_NAME-$VERSION.tar.gz"

# Build SRPM
echo "Building SRPM..."
rpmbuild -bs "$SPEC_FILE" --define "_topdir $HOME/rpmbuild"

# Show result
echo "SRPM created:"
ls -la ~/rpmbuild/SRPMS/"$PACKAGE_NAME-$VERSION-$RELEASE"*.src.rpm

# Copy to output directory if specified
if [ -n "$1" ]; then
    cp ~/rpmbuild/SRPMS/"$PACKAGE_NAME-$VERSION-$RELEASE"*.src.rpm "$1/"
    echo "SRPM copied to $1/"
fi