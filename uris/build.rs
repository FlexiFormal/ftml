use rustc_version::{Channel, version_meta};

fn main() {
    // Set cfg flags depending on release channel
    let channel = match version_meta().expect("version_meta").channel {
        Channel::Stable => "CHANNEL_STABLE",
        Channel::Beta => "CHANNEL_BETA",
        Channel::Nightly => "CHANNEL_NIGHTLY",
        Channel::Dev => "CHANNEL_DEV",
    };
    println!("cargo:rustc-cfg={channel}");
}
