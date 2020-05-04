# Development

## Codebase

### Design Decisions

#### Why use Tokio over X?

Hop uses `tokio` as an async executor. Initially in early development, it used
`async-std` and when `smol` was released an attempt at transitioning to it was
made. When attempting to use `smol` after it came out, there were a number of
oddities with TCP streams that couldn't be pinned down. Unlike the move to
`smol`, the move to `tokio` was flawless and worked out of the gate. Tokio has
been around the block for a while, it's fast, relatively mature, the runtime is
highly configurable, and importantly for Hop it's lean. The only thing we need
is basic TCP capability and I/O.

With only the minimal features required selected from both, transitioning from
`async-std` to `tokio` nearly halved the total dependency count from 59 to 31,
which is important for the project for reasons like depending on a smaller
number of maintainers, potential distro packaging, and a smaller surface of
potential security issues. Additionally, a clean hot cache -O3 compile time was
reduced from 45.73 seconds to 31.94.
