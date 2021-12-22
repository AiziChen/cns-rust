
pub fn xor_cipher<'a>(data: &'a mut [u8], secret: &str, sub_index: usize) -> &'a [u8] {
    let secret = secret.as_bytes();
    let s_len = secret.len();
    for (i, b) in data.iter_mut().enumerate() {
        let rem = (sub_index + i) % s_len;
        *b ^= secret[rem] | rem as u8;
    }
    return data;
}

pub unsafe fn decrypt_host(host: &mut str) -> Option<String> {
    return match base64::decode(host) {
        Ok(mut host) => {
            let de_host = xor_cipher(&mut host[..], "quanyec", 0);
            Some(String::from_utf8_lossy(&de_host[0..de_host.len() - 1]).to_string())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            None
        }
    };
}
