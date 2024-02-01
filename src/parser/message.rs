const CONTENT_LENGTH: [u8; 16] = [
    67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32,
];

#[derive(Debug)]
pub struct Message {
    pub payload: String,
}

impl Message {
    fn parse(buffer: &[u8]) -> Option<Vec<Message>> {
        for i in 0..buffer.len() {
            if buffer[i..i + (CONTENT_LENGTH.len())] == CONTENT_LENGTH {
                let mut idx = i + CONTENT_LENGTH.len();
                let mut s = String::new();
                while buffer[idx].is_ascii_digit() {
                    s.push(buffer[idx] as char);
                    idx += 1;
                }
                let len: usize = s.parse().expect("cannot parse content length");
                idx += 2; //two \n after the Content-Length
                let msg = Message {
                    payload: String::from_utf8_lossy(&buffer[idx..idx + len]).to_string(),
                };
                return Some(vec![msg]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn parse_simple_message() {
        let msg = r#"Content-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":3}"#;
        let result = Message::parse(msg.as_bytes());
        assert!(result.is_some());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
    }
}
