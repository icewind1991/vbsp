use main_error::MainError;
use obj::{Group, IndexTuple, Obj, ObjData, Object, SimplePolygon};

fn main() -> Result<(), MainError> {
    let mut args = std::env::args();
    let _ = args.next();
    let data = std::fs::read(args.next().expect("No demo file provided"))?;
    let bsp = vbsp::Bsp::read(&data)?;

    let vertices: Vec<_> = bsp
        .vertices
        .iter()
        .map(|vertex| <[f32; 3]>::from(vertex.position))
        .collect();

    let world_model = bsp.models().next().unwrap();

    let world_polygons = world_model
        .faces()
        .filter(|face| face.is_visible())
        .map(|face| {
            face.vertex_indexes()
                .map(|vertex_index| IndexTuple(vertex_index as usize, None, None))
                .collect()
        })
        .map(SimplePolygon)
        .collect();

    let world_object = Object {
        name: "".to_string(),
        groups: vec![Group {
            name: "".to_string(),
            index: 0,
            material: None,
            polys: world_polygons,
        }],
    };

    let obj_data = ObjData {
        position: vertices,
        texture: Vec::new(),
        normal: Vec::new(),
        objects: vec![world_object],
        material_libs: Vec::new(),
    };

    let obj = Obj {
        data: obj_data,
        path: Default::default(),
    };

    obj.save("out.obj")?;

    Ok(())
}
