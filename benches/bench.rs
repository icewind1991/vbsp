#![feature(test)]

extern crate test;

mod benches {
    use bsp::Bsp;
    use test::Bencher;

    const MAP_BYTES: &[u8] = include_bytes!("../test.bsp");

    #[bench]
    fn from_bytes(b: &mut Bencher) {
        use std::io::Cursor;

        b.iter(|| {
            Bsp::read(&mut Cursor::new(MAP_BYTES)).unwrap();
        });
    }

    #[bench]
    fn leaf_at(b: &mut Bencher) {
        use std::io::Cursor;

        let bsp = Bsp::read(&mut Cursor::new(MAP_BYTES)).unwrap();

        b.iter(|| {
            test::black_box(bsp.leaf_at(test::black_box([0., 0., 0.])));
        });
    }
}
