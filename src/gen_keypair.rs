use rand::rngs::OsRng;
use std::fs;
use x25519_dalek::{PublicKey, StaticSecret};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    fs::write("recipient_secret.key", secret.to_bytes())?;
    fs::write("recipient_public.key", public.as_bytes())?;
    println!("Keys written to recipient_secret.key and recipient_public.key");
    Ok(())
}
