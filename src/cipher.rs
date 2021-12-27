
pub fn xor_cipher(data: &mut [u8], secret: &str, sub_index: usize) -> usize {
    let secret = secret.as_bytes();
    let s_len = secret.len();
    let mut rem = sub_index;
    for (i, b) in data.iter_mut().enumerate() {
        rem = (sub_index + i) % s_len;
        *b ^= secret[rem] | rem as u8;
    }
    return rem + 1;
}

pub unsafe fn decrypt_host(host: &mut str) -> Option<String> {
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
