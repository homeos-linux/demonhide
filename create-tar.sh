#!/bin/bash
cargo vendor
VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f2)
tar czf "demonhide-$VERSION.tar.gz" --transform "s,^,demonhide-$VERSION/," \
    Cargo.toml Cargo.lock src vendor .cargo