fn main() -> Result<(), winver::Error> {
    let [major, minor, patch, build] = winver::get_file_version_info(std::env::args_os().nth(1).expect("missing path"))?;
    println!("{major}.{minor}.{patch}.{build}");
    Ok(())
}
