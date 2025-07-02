mod extensions;
mod extract;
mod format;

fn main() {
    let extracted = extract::extract();

    extensions::generate(&extracted).unwrap();
    format::generate(&extracted).unwrap();
}
