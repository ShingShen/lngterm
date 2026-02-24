// Linux Only //
fn main() {
    cc::Build::new()
    .files([
        "native/serial.c",
    ])
    .include("native")
    .compile("serialterm");
}
// Linux Only //