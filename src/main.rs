mod constants;
mod error;
mod prelude;
mod utils;

use prelude::*;

fn main() -> Result<()> {
    let source_path = std::path::Path::new(constants::SOURCE);

    loop {
        match std::fs::read_dir(source_path) {
            Ok(files) => utils::handle(files, constants::TARGET)?,
            Err(e) => eprintln!("{} {}", constants::READ_DIR_ERROR, e),
        }
        std::thread::sleep(std::time::Duration::from_secs(constants::SLEEP_INTERVAL));
    }
}
