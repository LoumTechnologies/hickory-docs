//! TLS ClientHello SNI extraction.
//!
//! Manual parser that extracts the Server Name Indication hostname from
//! a raw TLS ClientHello message without depending on a full TLS stack.

/// Extract the SNI hostname from a TLS ClientHello message.
///
/// Returns `None` if the data is not a valid ClientHello or doesn't
/// contain an SNI extension.
pub fn extract_sni(data: &[u8]) -> Option<String> {
    // TLS record header: ContentType(1) + Version(2) + Length(2)
    if data.len() < 5 {
        return None;
    }
    if data[0] != 0x16 {
        return None; // Not a Handshake record
    }

    let record_len = u16::from_be_bytes([data[3], data[4]]) as usize;
    if data.len() < 5 + record_len {
        return None;
    }

    let hs = &data[5..5 + record_len];

    // Handshake header: Type(1) + Length(3)
    if hs.is_empty() || hs[0] != 0x01 {
        return None; // Not ClientHello
    }
    if hs.len() < 4 {
        return None;
    }

    let hs_len = u24_to_usize(&hs[1..4]);
    if hs.len() < 4 + hs_len {
        return None;
    }
    let ch = &hs[4..4 + hs_len];

    // ClientHello: Version(2) + Random(32) + SessionID(var) + CipherSuites(var) + Compression(var) + Extensions(var)
    if ch.len() < 34 {
        return None;
    }
    let mut pos = 34; // skip version + random

    // Session ID
    if pos >= ch.len() {
        return None;
    }
    let session_id_len = ch[pos] as usize;
    pos += 1 + session_id_len;

    // Cipher Suites
    if pos + 2 > ch.len() {
        return None;
    }
    let cs_len = u16::from_be_bytes([ch[pos], ch[pos + 1]]) as usize;
    pos += 2 + cs_len;

    // Compression methods
    if pos >= ch.len() {
        return None;
    }
    let comp_len = ch[pos] as usize;
    pos += 1 + comp_len;

    // Extensions
    if pos + 2 > ch.len() {
        return None;
    }
    let ext_len = u16::from_be_bytes([ch[pos], ch[pos + 1]]) as usize;
    pos += 2;

    let ext_end = pos + ext_len;
    if ext_end > ch.len() {
        return None;
    }

    // Walk extensions looking for SNI (type 0x0000)
    while pos + 4 <= ext_end {
        let ext_type = u16::from_be_bytes([ch[pos], ch[pos + 1]]);
        let ext_data_len = u16::from_be_bytes([ch[pos + 2], ch[pos + 3]]) as usize;
        pos += 4;

        if ext_type == 0x0000 {
            // SNI extension
            return parse_sni_extension(&ch[pos..pos + ext_data_len]);
        }

        pos += ext_data_len;
    }

    None
}

/// Parse the SNI extension data to extract the hostname.
fn parse_sni_extension(data: &[u8]) -> Option<String> {
    if data.len() < 2 {
        return None;
    }
    let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + list_len {
        return None;
    }

    let mut pos = 2;
    let end = 2 + list_len;

    while pos + 3 <= end {
        let name_type = data[pos];
        let name_len = u16::from_be_bytes([data[pos + 1], data[pos + 2]]) as usize;
        pos += 3;

        if name_type == 0 && pos + name_len <= end {
            // Host name type
            return String::from_utf8(data[pos..pos + name_len].to_vec()).ok();
        }
        pos += name_len;
    }

    None
}

fn u24_to_usize(b: &[u8]) -> usize {
    ((b[0] as usize) << 16) | ((b[1] as usize) << 8) | (b[2] as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal TLS 1.2 ClientHello with SNI for testing.
    fn build_client_hello(hostname: &str) -> Vec<u8> {
        let hostname_bytes = hostname.as_bytes();

        // SNI extension data
        let sni_name_len = hostname_bytes.len();
        let sni_list_len = 3 + sni_name_len; // type(1) + len(2) + name
        let sni_ext_len = 2 + sni_list_len; // list_len(2) + list

        let mut sni_ext = Vec::new();
        sni_ext.extend_from_slice(&0x0000u16.to_be_bytes()); // SNI extension type
        sni_ext.extend_from_slice(&(sni_ext_len as u16).to_be_bytes()); // extension data length
        sni_ext.extend_from_slice(&(sni_list_len as u16).to_be_bytes()); // server name list length
        sni_ext.push(0x00); // host name type
        sni_ext.extend_from_slice(&(sni_name_len as u16).to_be_bytes());
        sni_ext.extend_from_slice(hostname_bytes);

        let extensions_len = sni_ext.len();

        // ClientHello body
        let mut ch = Vec::new();
        ch.extend_from_slice(&[0x03, 0x03]); // TLS 1.2
        ch.extend_from_slice(&[0u8; 32]); // random
        ch.push(0); // session ID length = 0
        ch.extend_from_slice(&2u16.to_be_bytes()); // cipher suites length
        ch.extend_from_slice(&[0x00, 0x2F]); // TLS_RSA_WITH_AES_128_CBC_SHA
        ch.push(1); // compression methods length
        ch.push(0); // null compression
        ch.extend_from_slice(&(extensions_len as u16).to_be_bytes());
        ch.extend_from_slice(&sni_ext);

        // Handshake header
        let ch_len = ch.len();
        let mut hs = Vec::new();
        hs.push(0x01); // ClientHello
        hs.push((ch_len >> 16) as u8);
        hs.push((ch_len >> 8) as u8);
        hs.push(ch_len as u8);
        hs.extend_from_slice(&ch);

        // TLS record header
        let hs_len = hs.len();
        let mut record = Vec::new();
        record.push(0x16); // Handshake
        record.extend_from_slice(&[0x03, 0x01]); // TLS 1.0 (record version)
        record.extend_from_slice(&(hs_len as u16).to_be_bytes());
        record.extend_from_slice(&hs);

        record
    }

    #[test]
    fn extract_sni_from_client_hello() {
        let data = build_client_hello("api.openai.com");
        let sni = extract_sni(&data).unwrap();
        assert_eq!(sni, "api.openai.com");
    }

    #[test]
    fn returns_none_for_non_tls() {
        let data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert!(extract_sni(data).is_none());
    }

    #[test]
    fn returns_none_for_too_short() {
        assert!(extract_sni(&[]).is_none());
        assert!(extract_sni(&[0x16, 0x03, 0x01]).is_none());
    }
}
