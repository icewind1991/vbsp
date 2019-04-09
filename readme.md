# Example usage

(Assuming that the pk3 file is unzipped locally)

```rust
extern crate quake3_loader;
use std::fs::{File};

fn main() {
    let mut file = File::open("pak0/maps/q3dm1.bsp").unwrap();
    let bsp = quake3_loader::read_bsp(&file);
    println!("{:?}", bsp);
}
```
