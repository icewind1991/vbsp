#![feature(test)]

extern crate test;

mod benches {
    use test::Bencher;
    use vbsp::{Bsp, Vector};

    const MAP_BYTES: &[u8] = include_bytes!("../koth_bagel_rc2a.bsp");

    #[bench]
    fn from_bytes(b: &mut Bencher) {
        b.iter(|| {
            Bsp::read(&MAP_BYTES).unwrap();
        });
    }

    #[bench]
    fn leaf_at(b: &mut Bencher) {
        let bsp = Bsp::read(&MAP_BYTES).unwrap();

        b.iter(|| {
            test::black_box(bsp.leaf_at(test::black_box(Vector {
                x: 0.,
                y: 0.,
                z: 0.,
            })));
        });
    }
}
