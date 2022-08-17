pub fn xor_cipher(data: &mut [u8], secret: &str, sub_index: usize) -> usize {
    let secret = secret.as_bytes();
    let s_len = secret.len();
    let mut rem = sub_index;
    for (i, b) in data.iter_mut().enumerate() {
        rem = (sub_index + i) % s_len;
        *b ^= secret[rem] | (rem as u8);
    }
    return rem + 1;
}

pub fn decrypt_host(host: &mut str) -> Option<String> {
    return match base64::decode(host) {
        Ok(mut host) => {
            xor_cipher(&mut host[..], "quanyec", 0);
            Some(String::from_utf8_lossy(&host[0..host.len() - 1]).to_string())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            None
        }
    };
}

#[test]
fn xor_cipher_test() {
    unsafe {
        let mut raw_msg = String::from("Hi from there");
        let buf = raw_msg.as_bytes_mut();
        let mut rem = 0;
        rem = xor_cipher(&mut buf[..2], "secret1", rem);
        println!("rem: {}", rem);
        rem = xor_cipher(&mut buf[2..], "secret1", rem);
        println!("rem: {}", rem);

        let mut drem = 0;
        drem = xor_cipher(&mut buf[..2], "secret1", drem);
        println!("drem: {}", drem);
        drem = xor_cipher(&mut buf[2..], "secret1", drem);
        println!("drem: {}", drem);
        assert_ne!(String::from_utf8_lossy(&buf).to_string(), raw_msg);
    }
}
