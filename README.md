# rust-tradier
Someday it may become a general client for the [Tradier API](https://documentation.tradier.com/) in Rust, but for now, it's just the functionality I need for a project.

# Design goals

- Performance
- Zero copy. There are cases where copying is good, but only the user of this library would know that, not this lib.
- When structs are created, they should be packed/aligned optimally
- This is intended to be a general purpose implementation, but it is intended to be used inside an optimized thread-per-core approach with appropriate channels to other cores and such. Ideally it doesn't implement or depend on those things itself, but it must be implemented such that it can plugin to an implementation that does.
