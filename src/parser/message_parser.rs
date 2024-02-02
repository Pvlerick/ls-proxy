use super::Message;

const CONTENT_LENGTH: [u8; 16] = [
    67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32,
];

#[derive(Debug)]
pub struct MessageParser {
    leftover: Option<Vec<u8>>,
}

impl MessageParser {
    fn new() -> Self {
        MessageParser { leftover: None }
    }

    fn parse(&mut self, buffer: &[u8]) -> Vec<Message> {
        let mut parsed = Vec::<Message>::new();
        // TODO Check if this could be done without memcopy...
        let working_data = [&self.leftover.take().unwrap_or_default(), buffer].concat();
        let mut index = 0;
        while index < working_data.len() {
            let candidate_end = index + CONTENT_LENGTH.len();
            if candidate_end < working_data.len()
                && working_data[index..candidate_end] == CONTENT_LENGTH
            {
                let mut idx = index + CONTENT_LENGTH.len();
                let mut content_length: usize = 0;
                while working_data[idx].is_ascii_digit() {
                    content_length =
                        (content_length * 10) + Into::<usize>::into(working_data[idx] - 48);
                    idx += 1;
                }
                idx += 2; //two \n after the Content-Length
                let msg = Message {
                    payload: String::from_utf8_lossy(&working_data[idx..idx + content_length])
                        .to_string(),
                };
                parsed.push(msg);
            } else {
                self.leftover = Some(working_data[index..].to_vec());
            }
            index += 1;
        }
        return parsed;
    }
}

#[cfg(test)]
mod tests {
    use super::MessageParser;

    #[test]
    fn parse_empty() {
        let mut sut = MessageParser::new();
        let res = sut.parse("".as_bytes());
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn parse_junk() {
        let mut sut = MessageParser::new();
        let res = sut.parse(
            r#"Content-foo 45

and then some"#
                .as_bytes(),
        );
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn parse_one_message() {
        let mut sut = MessageParser::new();
        let msg = r#"Content-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":3}"#;
        let res = sut.parse(msg.as_bytes());
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
    }

    #[test]
    fn parse_two_messages() {
        let mut sut = MessageParser::new();
        let msg = r#"Content-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":3}
"Content-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":4}"#;
        let res = sut.parse(msg.as_bytes());
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert_eq!(
            res[1].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":4}"#
        );
    }

    #[test]
    fn parse_one_message_surrounded_by_junk() {
        let mut sut = MessageParser::new();
        let msg = r#"foo barContent-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":3}and some
more junk"#;
        let res = sut.parse(msg.as_bytes());
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert!(sut.leftover.is_some());
        assert_eq!(sut.leftover.unwrap().len(), 17);
    }

    #[test]
    fn parse_two_messages_surrounded_by_junk() {
        let mut sut = MessageParser::new();
        let msg = r#"some junk
Content-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":3}and more some inbetween
Content-Length: 44

{"jsonrpc":"2.0","method":"shutdown","id":4}and even some more at the end"#;
        let res = sut.parse(msg.as_bytes());
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert_eq!(
            res[1].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":4}"#
        );
        assert_eq!(sut.leftover.unwrap().len(), 29);
    }
}
