#[test]
fn b64() {
    fn t(i: &'static str, o: &'static str) {
        let mut x = vec![];
        encode(i.as_bytes(), &mut x).unwrap();
        assert_eq!(x, o.as_bytes());
    }

    t("Hello World!", "SGVsbG8gV29ybGQh");
    t("Hello World", "SGVsbG8gV29ybGQ=");
}

pub fn encode(mut input: &[u8], output: &mut impl std::io::Write) -> std::io::Result<()> {
    const Α: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    while let [a, b, c, rest @ ..] = input {
        let α = ((*a as usize) << 16) | ((*b as usize) << 8) | *c as usize;
        output.write_all(&[
            Α[α >> 18],
            Α[(α >> 12) & 0x3F],
            Α[(α >> 6) & 0x3F],
            Α[α & 0x3F],
        ])?;
        input = rest;
    }
    if !input.is_empty() {
        let mut α = (input[0] as usize) << 16;
        if input.len() > 1 {
            α |= (input[1] as usize) << 8;
        }
        output.write_all(&[Α[α >> 18], Α[α >> 12 & 0x3F]])?;
        if input.len() > 1 {
            output.write_all(&[Α[α >> 6 & 0x3f]])?;
        } else {
            output.write_all(&[b'='])?;
        }
        output.write_all(&[b'='])?;
    }
    Ok(())
}

pub const fn size(of: &[u8]) -> usize {
    let use_pad = of.len() % 3 != 0;
    if use_pad {
        4 * (of.len() / 3 + 1)
    } else {
        4 * (of.len() / 3)
    }
}
