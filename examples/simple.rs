use std::time::{Duration, Instant};

use mini_rust_runtime::Delay;
use mini_rust_runtime::MiniRust;

fn main() {
    let mini_rust = MiniRust::new();
    mini_rust.spawn(async {
        Delay {
            when: Instant::now() + Duration::from_secs(5),
        }
        .await;
        println!("hello mini-rust-runtime!");
    });
    mini_rust.spawn(async {
        Delay {
            when: Instant::now() + Duration::from_secs(10),
        }
        .await;
        println!("hello fys!");
    });
    mini_rust.run();
}
