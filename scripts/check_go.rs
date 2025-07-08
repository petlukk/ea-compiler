use std::process::Command;

fn main() {
    let available = Command::new("go")
        .arg("version")
        .output()
        .map( < /dev/null | output| output.status.success())
        .unwrap_or(false);
    println\!("Go available: {}", available);
}
