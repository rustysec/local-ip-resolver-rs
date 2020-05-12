local-ip-resolver
=================

simple interface to get the current ip address used
to communicate to a remote host.

# Example

```rust
use local_ip_resolver::for_host;
let request = std::env::args().nth(1).unwrap();
println!("Using {} for {}", for_host(&request).unwrap(), request);
```

