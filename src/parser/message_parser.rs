use super::Message;

const CONTENT_LENGTH: [u8; 16] = [
    67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32,
];

#[derive(Debug)]
pub struct MessageParser {
    leftover: [u8],
}

impl MessageParser {
    fn parse(&mut self, buffer: &[u8]) -> Option<Vec<Message>> {
        // TODO Check if this could be done without memcopy?
        let working_data = [&self.leftover, buffer].concat();
        for i in 0..working_data.len() {
            let candidate_end = i + CONTENT_LENGTH.len();
            if candidate_end < working_data.len()
                && working_data[i..candidate_end] == CONTENT_LENGTH
            {
                let mut idx = i + CONTENT_LENGTH.len();
                let mut s = String::new();
                while working_data[idx].is_ascii_digit() {
                    s.push(working_data[idx] as char);
                    idx += 1;
                }
                let len: usize = s.parse().expect("cannot parse content length");
                idx += 2; //two \n after the Content-Length
                let msg = Message {
                    payload: String::from_utf8_lossy(&working_data[idx..idx + len]).to_string(),
                };
                return (Some(msg), idx + len);
            }
        }
        (None, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::MessageParser;

    #[test]
    fn parse_empty() {
        let (message, last_index) = MessageParser::parse("".as_bytes(), 0);
        assert!(message.is_none());
        assert_eq!(last_index, 0);
    }

    #[test]
    fn parse_junk() {
        let (message, last_index) = MessageParser::parse(
            r#"Content-foo 45

and then some"#
                .as_bytes(),
            0,
        );
        assert!(message.is_none());
        assert_eq!(last_index, 0);
    }

    #[test]
    fn parse_simple_message() {
        let msg = r#"Content-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":3}"#;
        let (message, last_index) = MessageParser::parse(msg.as_bytes(), 0);
        assert!(message.is_some());
        assert_eq!(
            message.unwrap().payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert_eq!(last_index, 64);
    }

    #[test]
    fn parse_simple_message_surrounded_by_junk() {
        let msg = r#"foo barContent-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":3}and some
more junk"#;
        let (message, last_index) = MessageParser::parse(msg.as_bytes(), 0);
        assert!(message.is_some());
        assert_eq!(
            message.unwrap().payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert_eq!(last_index, 71);
    }
}
