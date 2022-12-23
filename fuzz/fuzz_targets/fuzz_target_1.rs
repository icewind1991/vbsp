#![no_main]

use libfuzzer_sys::fuzz_target;

fn fuzz(data: &[u8]) {
    if let Some(bsp) = vbsp::Bsp::read(data).ok() {
        let verts: Vec<_> = bsp
            .vertices
            .iter()
            .map(|vertex| vertex.position.x)
            .collect();
        assert!(verts.len() > 1);
    }
}

fuzz_target!(|data: &[u8]| { fuzz(data) });
