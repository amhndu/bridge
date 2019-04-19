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


