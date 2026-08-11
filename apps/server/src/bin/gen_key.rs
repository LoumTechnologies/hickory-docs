//! `just gen-key` — mint a `KEY_ENCRYPTION_KEY`.
//!
//! Separate from `main.rs` for the same reason as `print_openapi`: an
//! operator setting a deployment up for the first time has no database, no
//! configuration, and nothing running. Needing any of that to produce the
//! value that *unblocks* configuration would be a circle.
//!
//! Prints the assignment line, ready to paste into `.env` or hand to the
//! platform's secret store.
use hickory_server::keyvault::KeyVault;

fn main() {
    println!("KEY_ENCRYPTION_KEY={}", KeyVault::generate_base64());
    eprintln!(
        "\nThis key encrypts every account's stored provider API key.\n\
         - Set it once per environment and keep it: replacing it makes every \
         stored key unreadable, and each account has to paste its key again.\n\
         - Never commit it. On Fly: `fly secrets set KEY_ENCRYPTION_KEY=…`\n\
         - Local dev: append the line above to .env"
    );
}
