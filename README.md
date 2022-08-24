# Rust wrapper for [dbcppp](https://github.com/xR3b0rn/dbcppp)

## Dev setup

* Install build-essential, cmake, clang, and ninja
* `git clone`
* `git submodule update --init --recurse`
* `(mkdir -p dbcppp/cmake-build-debug && cd dbcppp/cmake-build-debug && cmake ../ && ninja && bin/libdbcppp_Test)`
* `cargo test`