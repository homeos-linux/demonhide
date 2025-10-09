#!/bin/bash
cargo vendor
tar czf demonhide-0.1.1.tar.gz --transform 's,^,demonhide-0.1.1/,' \
  Cargo.toml Cargo.lock src vendor .cargo