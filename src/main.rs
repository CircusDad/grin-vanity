use clap::Parser;
use grin_core::global::ChainTypes;
use grin_keychain::keychain::ExtKeychain;
use grin_keychain::Identifier;
use grin_keychain::Keychain;
use grin_wallet_libwallet::SlatepackAddress;
use rand::Rng;
use std::process;
use std::thread;
use std::time::Instant;

mod args;
use args::Args;

// Measures how many s elapsed since the given instant
fn time_since(instant: Instant) -> f64 {
    (instant.elapsed().as_nanos() as f64) / 1_000_000_000f64
}

fn main() {
    let args = Args::parse();

    // Clone the pattern and suffix before the loop
    let pattern = args.prefix.clone();
    let suffix = args.suffix.clone();

    println!(
        "Searching for prefix pattern {} and suffix pattern {}",
        pattern,
        suffix.as_deref().unwrap_or("None")
    );
    println!("Using {} threads", args.threads);

    // Validate prefix pattern
    if !pattern.starts_with("grin1") {
        println!("Pattern needs to start with grin1");
        process::exit(0x1);
    } else if pattern[5..]
        .to_string()
        .chars()
        .any(|c| c == '1' || c == 'i' || c == 'o' || c == 'b')
    {
        println!("Invalid prefix pattern");
        println!("Valid characters are: acdefghjklmnpqrstuvwxyz023456789");
        std::process::exit(1);
    }

    // Validate suffix pattern (if present)
    if let Some(ref sfx) = suffix {
        if sfx.chars().any(|c| c == '1' || c == 'i' || c == 'o' || c == 'b') {
            println!("Invalid suffix pattern");
            println!("Valid characters are: acdefghjklmnpqrstuvwxyz023456789");
            std::process::exit(1);
        }
    }

    let mut handles = Vec::new();

    // Spawn worker threads
    for thread_id in 0..args.threads {
        let pattern = pattern.clone(); // Clone pattern inside the loop for each thread
        let suffix = suffix.clone();   // Clone suffix inside the loop for each thread
        let refresh_interval = args.interval;
        let parent_key_id = Identifier::from_bytes(&[
            2, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00,
        ]);

        let t = thread::spawn(move || {
            let mut i = 0;
            let start_time = Instant::now();
            let mut stats_timer = Instant::now();
            grin_core::global::set_local_chain_type(ChainTypes::Mainnet);

            loop {
                let loop_time = Instant::now();
                let bytes: [u8; 32] = rand::thread_rng().gen();

                // From seed
                let keychain = ExtKeychain::from_seed(&bytes, false).unwrap();
                let sec_addr_key = grin_wallet_libwallet::address::address_from_derivation_path(
                    &keychain,
                    &parent_key_id,
                    0,
                )
                .unwrap();
                let slatepack_address = SlatepackAddress::try_from(&sec_addr_key).unwrap();

                // Check the prefix and suffix
                let address_str = slatepack_address.to_string();
                let prefix_match = address_str.starts_with(&pattern);
                let suffix_match = match &suffix {
                    Some(sfx) => address_str.ends_with(sfx),
                    None => true, // No suffix pattern means we only match the prefix
                };

                if thread_id == 0 && time_since(stats_timer) > refresh_interval as f64 {
                    let pattern_length = pattern.len() - 5; // Subtracting 'grin1' prefix length
                    let total_length = match &suffix {
                        Some(sfx) => pattern_length + sfx.len(), // Add suffix length if provided
                        None => pattern_length, // No suffix, just use pattern length
                    };

                    let num_of_patterns = 33_u64.pow(total_length as u32);
                    let iteration_time = time_since(loop_time);
                    let keys_per_second = (1. / iteration_time) * args.threads as f64;
                    let eta = (iteration_time * num_of_patterns as f64) / args.threads as f64;

                    print!("{:.2} keys/s ", keys_per_second);

                    if eta < 60. {
                        println!("eta: {:.2}s", eta as usize);
                    } else if eta < 3600. {
                        println!("eta: {:.2}min", eta / 60.);
                    } else if eta < 86400. {
                        println!("eta: {:.2}h", eta / 3600.);
                    } else if eta < 2073600. {
                        println!("eta: {:.2}d", eta / 86400.);
                    } else {
                        println!("eta: {:.2}y", eta / 2073600.);
                    }
                    stats_timer = Instant::now();
                }

                if prefix_match && suffix_match {
                    println!(
                        "\nFound address: {} \nWith Seed:     {} \n{} keys in {} seconds",
                        slatepack_address,
                        grin_keychain::mnemonic::from_entropy(&bytes).unwrap(),
                        i * args.threads,
                        time_since(start_time)
                    );
                    process::exit(0x0);
                }

                i += 1;
            }
        });
        handles.push(t);
    }

    // Wait for threads to finish
    for handle in handles {
        handle.join().expect("Error joining worker thread");
    }
}
