#!/bin/bash
cargo vendor
VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f2)

# Create .cargo/config.toml with vendoring configuration
mkdir -p .cargo
cat > .cargo/config.toml <<'EOF'
[build]
# Use incremental compilation for faster CI builds
incremental = true

[profile.dev]
# Faster linking for development
split-debuginfo = "unpacked"

[profile.release]
# Optimize for binary size and performance
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[net]
# Use sparse registry for faster dependency resolution
git-fetch-with-cli = true

[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

tar czf "demonhide-$VERSION.tar.gz" --transform "s,^,demonhide-$VERSION/," \
    Cargo.toml Cargo.lock src vendor .cargo README.md LICENSE
rm -rf .cargo vendor
git add .
git commit -m "Update source tarball to version $VERSION"
git tag -d "demonhide-$VERSION-1" 2>/dev/null || true
git tag "demonhide-$VERSION-1"
git push --tags origin main --force