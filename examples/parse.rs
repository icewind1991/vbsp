fn main() -> Result<(), vbsp::BspError> {
    let mut args = std::env::args();
    let _ = args.next();
    let data = std::fs::read(args.next().unwrap())?;
    let _ = vbsp::Bsp::read(&data)?;

    Ok(())
}
