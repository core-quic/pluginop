# PluginOp: Pluginizable Operations

[![Test](https://github.com/core-quic/pluginop/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/core-quic/pluginop/actions/workflows/rust.yml?query=branch%main)
[![Benchmark](https://github.com/core-quic/pluginop/actions/workflows/benchmark.yml/badge.svg?branch=main)](https://github.com/core-quic/pluginop/actions/workflows/benchmark.yml?query=branch%main)
[![Test coverage](https://codecov.io/gh/core-quic/pluginop/branch/main/graph/badge.svg?token=22VU1757X3)](https://codecov.io/gh/core-quic/pluginop)

*One day, we will make the Internet great again. In the meantime, we struggle with engineering problems.*

PluginOp is a crate intending to make (Rust) network implementation seamlessly pluginizable.
The project consists in several sub-crates:

- [pluginop-common](https://github.com/core-quic/pluginop/tree/main/common): contains all the common structures (part of the standardized API) shared by both the plugins and the host implementation
- [pluginop](https://github.com/core-quic/pluginop/tree/main/lib): the main crate of this project, used by the host implementation to be pluginizable
- [pluginop-macro](https://github.com/core-quic/pluginop/tree/main/macro): contains macros to be used by the host implementation to pluginize its functions using one-liners
- [pluginop-mock](https://github.com/core-quic/pluginop/tree/main/mock): a mocking host implementation used to test and benchmark the whole project
- [pluginop-octets](https://github.com/core-quic/pluginop/tree/main/octets): a fork of the [quiche's octets crate](https://github.com/cloudflare/quiche/tree/master/octets) with support to raw pointer conversion
- [pluginop-rawptr](https://github.com/core-quic/pluginop/tree/main/rawptr): an abstraction over raw pointers
- [pluginop-wasm](https://github.com/core-quic/pluginop/tree/main/wasm): the crate offering an API to plugins

The [tests folder](https://github.com/core-quic/pluginop/tree/main/tests) contains plugins for tests and benchmarks purposes.


## Supported Core Implementations

- [core-quic](https://github.com/core-quic/quiche): Based on quiche, main core implementation
- [core-quinn](https://github.com/core-quic/quinn): Based on quinn, mostly integrated in the quinn-proto crate. Does not have full support (yet), but simple frame-based plugins work.


## Existing plugins

QUIC-specific plugins are available at the [core-quic-plugins](https://github.com/core-quic/core-quic-plugins) repo.
