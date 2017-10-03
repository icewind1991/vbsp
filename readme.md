# Example usage

(Assuming that the pk3 file is unzipped locally)

    extern crate quake3_loader;
    use std::fs::{File};
    use std::path::{Path};
    use std::io::{Read};
    fn main() {
        let path = Path::new("pak0/maps/q3dm1.bsp");
        let mut file = File::open(path).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let bsp = quake3_loader::read_bsp(&bytes);           
        println!("{:?}", bsp);
    }

