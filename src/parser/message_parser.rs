use super::Message;

const CONTENT_LENGTH: [u8; 16] = [
    67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32,
];

#[derive(Debug)]
pub struct MessageParser {
    leftover: Vec<u8>,
}

impl MessageParser {
    pub fn new() -> Self {
        MessageParser {
            leftover: Vec::<u8>::new(),
        }
    }

    pub fn parse(&mut self, buffer: &[u8]) -> Vec<Message> {
        let mut parsed = Vec::<Message>::new();
        // TODO Check if this could be done without memcopy...
        let working_data = [&self.leftover, buffer].concat();
        let mut last_msg_end = 0;
        let mut index = 0;
        while index < working_data.len() {
            let candidate_end = index + CONTENT_LENGTH.len();

            if candidate_end >= working_data.len() {
                self.leftover = working_data[last_msg_end..].to_vec();
                return parsed;
            }

            if working_data[index..candidate_end] == CONTENT_LENGTH {
                let mut idx = candidate_end;
                let mut content_length: usize = 0;
                while working_data[idx].is_ascii_digit() {
                    content_length =
                        (content_length * 10) + Into::<usize>::into(working_data[idx] - 48);
                    idx += 1;
                    if idx >= working_data.len() {
                        self.leftover = working_data[last_msg_end..].to_vec();
                        return parsed;
                    }
                }
                idx += 4; //two \n after the Content-Length
                if idx >= working_data.len() {
                    self.leftover = working_data[last_msg_end..].to_vec();
                    return parsed;
                }
                let message_end = idx + content_length;
                if message_end > working_data.len() {
                    self.leftover = working_data[last_msg_end..].to_vec();
                    return parsed;
                }

                let msg = Message {
                    payload: String::from_utf8_lossy(&working_data[idx..message_end]).to_string(),
                };
                parsed.push(msg);
                index = idx + content_length;
                last_msg_end = index;
            } else {
                index += 1;
            }
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
        let msg =
            "Content-Length: 44\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"id\":3}";
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
        let msg = "Content-Length: 44\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"id\":3}Content-Length: 44\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"id\":4}";
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
        let msg = "foo barContent-Length: 44\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"id\":3}and some\r\nmore junk";
        let res = sut.parse(msg.as_bytes());
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert_eq!(sut.leftover.len(), 19);
    }

    #[test]
    fn parse_two_messages_surrounded_by_junk() {
        let mut sut = MessageParser::new();
        let msg = "some junk\r\nContent-Length: 44\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"id\":3}and more some inbetween\r\n\r\nContent-Length: 44\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"id\":4}and even some more at the end";
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
        assert_eq!(sut.leftover.len(), 29);
    }

    #[test]
    fn parse_one_message_in_two_chunks() {
        let mut sut = MessageParser::new();
        let chunk_1 = "foo barContent-Length: 44\r\n\r\n{\"jsonrpc\":\"2.0\",\"met";
        let chunk_2 = "hod\":\"shutdown\",\"id\":3}and some\r\nmore junk";
        let res = sut.parse(chunk_1.as_bytes());
        assert_eq!(res.len(), 0);
        assert_eq!(sut.leftover.len(), 50);
        let res = sut.parse(chunk_2.as_bytes());
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert_eq!(sut.leftover.len(), 19);
    }

    #[test]
    fn parse_one_message_in_two_chunks_cut_in_middle_of_content_length() {
        let mut sut = MessageParser::new();
        let chunk_1 = "foo barContent-Length: 4";
        let chunk_2 = "4\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"id\":3}and some\r\nmore obvious junk";
        let res = sut.parse(chunk_1.as_bytes());
        assert_eq!(res.len(), 0);
        assert_eq!(sut.leftover.len(), 24);
        let res = sut.parse(chunk_2.as_bytes());
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].payload,
            r#"{"jsonrpc":"2.0","method":"shutdown","id":3}"#
        );
        assert_eq!(sut.leftover.len(), 27);
    }
}
