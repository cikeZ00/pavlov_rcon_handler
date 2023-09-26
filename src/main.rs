mod tools;

use crate::tools::pavlov_rcon;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a PavlovRCON instance.
    let rcon = pavlov_rcon::PavlovRCON::new("192.168.232.128", 9100, "123", 30);

    // Example: Sending multiple commands and waiting for responses.
    let commands = vec!["help", "help", "help"];
    let responses = rcon.multisend(&commands)?;

    for (index, response) in responses.iter().enumerate() {
        println!("Response {}: {:?}", index + 1, response);
    }

    let single_command = "GiveAll 0 ww2knife";
    let single_response = rcon.send(single_command)?;
    println!("Single: {}", single_response);



    Ok(())
}

