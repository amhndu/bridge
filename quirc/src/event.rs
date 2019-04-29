#[derive(Debug, PartialEq)]
pub enum Prefix<'a> {
    Server {
        host: &'a str,
    },
    User {
        nick: &'a str,
        username: Option<&'a str>,
        host: &'a str,
    },
}

#[derive(Debug, PartialEq)]
pub struct RawEvent<'a> {
    pub prefix: Option<Prefix<'a>>,
    pub command: &'a str,
    pub params: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub enum MessageTarget {
    Channel(String),
    User(String),
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Ping { server: String, server2: Option<String> },
    Welcome { server_message: String, host: String, server_created: String, server_info: String },
    ChannelJoined { topic: String },
    NewMessage { target: MessageTarget, message: String },

    // Errors
    NickFailure(String),
    JoinFailure(String),
    Unknown,
}
