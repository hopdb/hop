pub mod application;

mod error;

use async_std::task;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    task::block_on(application::run()).map_err(From::from)
}
