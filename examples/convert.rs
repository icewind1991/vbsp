use main_error::MainError;
use obj::{Group, IndexTuple, Obj, ObjData, Object, SimplePolygon};
use vbsp::TextureFlags;

fn main() -> Result<(), MainError> {
    let mut args = std::env::args();
    let _ = args.next();
    let data = std::fs::read(args.next().expect("No demo file provided"))?;
    let bsp = vbsp::Bsp::read(&data)?;

    let vertices = bsp
        .vertices
        .iter()
        .map(|vertex| <[f32; 3]>::from(&vertex.position))
        .collect();

    let polygons = bsp
        .original_faces()
        .filter(|face| {
            face.texture()
                .map(|texture| {
                    !texture.flags.intersects(
                        TextureFlags::LIGHT
                            | TextureFlags::SKY2D
                            | TextureFlags::SKY
                            | TextureFlags::SKIP
                            | TextureFlags::NODRAW,
                    )
                })
                .unwrap_or_default()
        })
        .map(|face| {
            SimplePolygon(
                face.vertex_indexes()
                    .map(|vertex| IndexTuple(vertex as usize, None, None))
                    .collect(),
            )
        })
        .collect();

    let obj_data = ObjData {
        position: vertices,
        texture: Vec::new(),
        normal: Vec::new(),
        objects: vec![Object {
            name: "".to_string(),
            groups: vec![Group {
                name: "".to_string(),
                index: 0,
                material: None,
                polys: polygons,
            }],
        }],
        material_libs: Vec::new(),
    };

    let obj = Obj {
        data: obj_data,
        path: Default::default(),
    };

    obj.save("out.obj")?;

    Ok(())
}
