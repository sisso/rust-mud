extern crate mud;

use std::path::Path;

use mud::game::container::Container;
use mud::game::loader::hocon_loader::HoconLoader;
use mud::game::loader::{Loader, Result};

#[test]
fn hocon_loader_test() -> Result<()> {
    let load = HoconLoader::load(Path::new("./data"))?;
    Ok(())
}
