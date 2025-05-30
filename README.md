# Moved to https://codeberg.org/icewind/vbsp

# VBSP

Rust parser for valve bsp files.

Mainly tested with Team Fortress 2 and Counter Strike Source files, other source games using bsp versions 19, 20 or
21 might also work.

Adding other sourcemod variants should be fairly straight forward and pull requests for them are welcome.

# Example usage

```rust
fn main() -> Result<(), vbsp::BspError> {
    let data = std::fs::read("maps/cp_steel.bsp")?;
    let bsp = vbsp::Bsp::read(&data)?;
    println!("{:?}", bsp);

    Ok(())
}
```

See [vbsp-to-gltf](https://github.com/icewind1991/vbsp-to-gltf) or [vbspviewer](https://github.com/icewind1991/vbspview)
for some more examples of how to use the bsp data.

## TODO

- [ ] smooth normals for displacements
- [ ] smooth normals for faces

## Credits

This project is adapted from the [quake bsp parser] and
wouldn't be possible without information from the [source engine wiki].

[quake bsp parser]: https://github.com/Vurich/bsp
[source engine wiki]: https://developer.valvesoftware.com/wiki/Source_BSP_File_Format
