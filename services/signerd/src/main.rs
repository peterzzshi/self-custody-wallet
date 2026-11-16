use std::io::{self, Read, Write};

fn main() {
    if let Err(e) = run() {
        eprintln!("signerd: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("signerd: skeleton (M0)");
    // M1: length-prefixed JSON frame loop over stdio.
    // First frame must be {type: "Unseal", passphrase: ...}; reject all
    // other frame types until the keystore is unsealed.
    let mut stdin = io::stdin().lock();
    let _ = stdin.read_to_end(&mut Vec::new())?;
    let mut stdout = io::stdout().lock();
    stdout.flush()?;
    Ok(())
}
