use imms::Client;
use std::error::Error;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::connect("0.0.0.0:14000").await?;

    // Increments only integers, sends an error if the key is a float.
    println!("New value: {}", client.increment_int("foo").await?);
    // Increments both integers and floats.
    println!("New value: {}", client.increment("foo").await?);
    println!("New value: {}", client.decrement_int("foo").await?);

    Ok(())
}
