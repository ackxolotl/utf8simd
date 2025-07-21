use utf8simd::from_utf8;

fn main() -> utf8simd::Result<()> {
    let data = b"hello world!";

    let str = from_utf8(data)?;
    println!("{str}");

    Ok(())
}
