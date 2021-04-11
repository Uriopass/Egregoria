use std::env;

fn main() {
    if let Ok(x) = std::fs::read("../assets/config.json") {
        let _ = std::fs::write(env::var("OUT_DIR").unwrap() + "/config.json", x);
    }
    println!("cargo:rerun-if-changed=../assets/config.json");
}
