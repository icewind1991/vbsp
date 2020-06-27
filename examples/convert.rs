use main_error::MainError;
use obj::{Group, IndexTuple, Obj, ObjData, Object, SimplePolygon};
use vbsp::TextureFlags;

fn main() -> Result<(), MainError> {
    let mut args = std::env::args();
    let _ = args.next();
    let data = std::fs::read(args.next().expect("No demo file provided"))?;
    let bsp = vbsp::Bsp::read(&data)?;

    let exclude_faces = TextureFlags::LIGHT
        | TextureFlags::SKY2D
        | TextureFlags::SKY
        | TextureFlags::WARP
        | TextureFlags::TRANS
        | TextureFlags::TRIGGER
        | TextureFlags::HINT
        | TextureFlags::SKIP
        | TextureFlags::NODRAW
        | TextureFlags::HITBOX;

    let vertices = bsp
        .vertices
        .iter()
        .map(|vertex| <[f32; 3]>::from(&vertex.position))
        .collect();

    let objects = bsp
        .models()
        .next() // only do "worldspawn" for now
        .into_iter()
        .map(|model| Group {
            name: "".to_string(),
            index: 0,
            material: None,
            polys: model
                .faces()
                .filter(|face| {
                    face.texture()
                        .map(|texture| !texture.flags.intersects(exclude_faces))
                        .unwrap_or_default()
                })
                .map(|face| {
                    SimplePolygon(
                        face.vertex_indexes()
                            .map(|vertex| IndexTuple(vertex as usize, None, None))
                            .collect(),
                    )
                })
                .collect(),
        })
        .map(|group| Object {
            name: "".to_string(),
            groups: vec![group],
        })
        .collect();

    let obj_data = ObjData {
        position: vertices,
        texture: Vec::new(),
        normal: Vec::new(),
        objects,
        material_libs: Vec::new(),
    };

    let obj = Obj {
        data: obj_data,
        path: Default::default(),
    };

    obj.save("out.obj")?;

    Ok(())
}
