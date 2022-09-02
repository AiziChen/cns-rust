use log::error;
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

pub fn decrypt_host(host: &str) -> Option<String> {
    return match base64::decode(host) {
        Ok(mut host) => {
            xor_cipher(&mut host, "quanyec", 0);
            Some(String::from_utf8_lossy(&host[0..host.len() - 1]).to_string())
        }
        Err(err) => {
            error!("Error: {}", err.to_string());
            None
        }
    };
}

#[test]
fn xor_cipher_test() {
    unsafe {
        const SECRET: &str = "secret1";
        let mut raw_msg = String::from("Hi from there");
        let buf = raw_msg.as_bytes_mut();
        let mut rem = 0;
        rem = xor_cipher(&mut buf[..2], SECRET, rem);
        assert_eq!(rem, 2);
        rem = xor_cipher(&mut buf[2..], SECRET, rem);
        assert_eq!(rem, SECRET.len() - 1);

        let mut drem = 0;
        drem = xor_cipher(&mut buf[..2], SECRET, drem);
        assert_eq!(drem, 2);
        drem = xor_cipher(&mut buf[2..], SECRET, drem);
        assert_eq!(drem, SECRET.len() - 1);

        assert_eq!(String::from_utf8_lossy(&buf).to_string(), raw_msg);
    }
}
