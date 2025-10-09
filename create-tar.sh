#!/bin/bash
cargo vendor
VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f2)
tar czf "demonhide-$VERSION.tar.gz" --transform "s,^,demonhide-$VERSION/," \
    Cargo.toml Cargo.lock src vendor .cargo
git add .
git commit -m "Update source tarball to version $VERSION"
git tag -d "v$VERSION" 2>/dev/null || true
git tag "demonhide-$VERSION-1"
git push --tags origin main --force