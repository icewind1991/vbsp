fn main() -> Result<(), vbsp::BspError> {
    let mut args = std::env::args();
    let _ = args.next();
    let data = std::fs::read(args.next().expect("No demo file provided"))?;
    let props = vbsp::Bsp::read(&data)?.entities;
    for prop in props.iter() {
        match prop.parse() {
            Ok(prop) => println!("{:#?}", prop),
            Err(e) => println!("Failed parsing {:#?}: {:#}", prop, e),
        }
    }

    Ok(())
}
