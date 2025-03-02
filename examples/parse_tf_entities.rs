use vbsp_entities_tf2::Entity;

fn main() -> Result<(), vbsp::BspError> {
    let mut args = std::env::args();
    let _ = args.next();
    let data = std::fs::read(args.next().expect("No demo file provided"))?;
    let bsp = vbsp::Bsp::read(&data)?;
    for prop in bsp.entities.iter() {
        match prop.parse::<Entity>() {
            Ok(prop) => {
                println!("{:#?}", prop)
            }
            Err(e) => {
                println!("Failed parsing {}: {:#}", prop.as_str(), e);
            }
        }
    }

    Ok(())
}
