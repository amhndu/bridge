use crate::event::{Prefix, RawEvent, Event};
use std::error::Error;

const SPACE: char = ' ';
const COLON: char = ':';
const CRLF: &str = "\r\n";

fn to_prefix_server(input: &str) -> Prefix {
    Prefix::Server { host: input }
}

fn is_specialalphanum(c: u8, special: &[u8]) -> bool {
    special.contains(&c) || nom::is_alphanumeric(c)
}

named!(host_parser<&[u8], &str>,
    map_res!(
        take_while!(|c| is_specialalphanum(c, b".")),
        std::str::from_utf8
    )
);

named!(nick_parser<&[u8], &str>,
    map_res!(
        take_while!(|c| is_specialalphanum(c, b"[]\\`_^{|}-")),
        std::str::from_utf8
    )
);

named!(server_prefix_parser<&[u8], Prefix>,
    map!(host_parser, to_prefix_server)
);

named!(user_prefix_parser<&[u8], Prefix>,
    do_parse!(
       nick: nick_parser                    >>
       username: opt!(
           preceded!(
               tag!(b"!"),
               map_res!(nom::alphanumeric, std::str::from_utf8)
           )
       )                                    >>
       tag!(b"@")                           >>
       host: host_parser                    >>
       (Prefix::User { nick, username, host } )
    )
);

named!(prefix_parser<&[u8], Prefix>,
    delimited!(
           char!(COLON),
           alt!(user_prefix_parser | server_prefix_parser ),
           char!(SPACE)
    )
);

named!(middle_params_parser<&[u8], Vec<&str> >,
    many_m_n!(0, 14,
        preceded!(
            char!(SPACE),
            map_res!(is_not!("\0\r\n :"), std::str::from_utf8)
        )
    )
);

named!(trailing_params_parser<&[u8], Option<&str> >,
    opt!(
        preceded!(
            alt!(tag!(" :") | tag!(" ")),
            map_res!(is_not!("\0\r\n"), std::str::from_utf8)
        )
    )
);

named!(params_parser<&[u8], Vec<&str> >,
   do_parse!(
       middle: middle_params_parser         >>
       trailing: trailing_params_parser     >>
       ({
           let mut params = middle;
           if let Some(trailing) = trailing {
               params.push(trailing);
           }

           params
       })
   )
);

named!(raw_command_parser<&[u8], &str>,
    map_res!(
        alt!(nom::digit | nom::alpha),
        std::str::from_utf8
    )
);

named!(raw_event_parser<&[u8], RawEvent>,
    do_parse!(
        prefix: opt!(prefix_parser)           >>
        command: raw_command_parser           >>
        params: params_parser                 >>
        tag!(CRLF)                            >>
        (RawEvent { prefix, command, params })
    )
);

fn parse_input<'a>(input: &'a [u8]) -> Event {
    let raw_event = raw_event_parser(input)
                        .map_err(|e| format!("Parsing Error: {}", e) );

    let event_res = || -> Result<Event, String> {
        const PARAM_ERROR: &str = "Expected parameter";

        let (_, raw_event) = raw_event?;
        Ok(match raw_event.command {
            Event::Ping.to_string() => {
                Event::Ping {
                    server:  raw_event.params.get(0).ok_or(PARAM_ERROR)?.to_string(),
                    server2: raw_event.params.get(1).map(|&s| s.to_string())
                }
            },
            cmd => return Err(format!("Unknown command: {}", cmd))
        })
    };

    match event_res() {
        Ok(event) => event,
        Err(error) => {
            error!("quirc: Error while reading incoming message. {}", error);
            Event::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_test() {
        assert_eq!(
            prefix_parser(b":irc.example.com end").unwrap(),
            (
                &b"end"[..],
                Prefix::Server {
                    host: "irc.example.com"
                }
            )
        );
        assert_eq!(
            prefix_parser(b":nick!on@example.com end").unwrap(),
            (
                &b"end"[..],
                Prefix::User {
                    nick: "nick",
                    username: Some("on"),
                    host: "example.com"
                }
            )
        );
        assert_eq!(
            prefix_parser(b":nick@example.com end").unwrap(),
            (
                &b"end"[..],
                Prefix::User {
                    nick: "nick",
                    username: None,
                    host: "example.com"
                }
            )
        );
    }

    #[test]
    fn params_test() {
        assert_eq!(
            middle_params_parser(b" first second third\r\n").unwrap(),
            (&b"\r\n"[..], vec!["first", "second", "third"])
        );
        assert_eq!(
            params_parser(b" first second third :last with spaces\r\n").unwrap(),
            (
                &b"\r\n"[..],
                vec!["first", "second", "third", "last with spaces"]
            )
        );
        assert_eq!(
            params_parser(b" first :last:with: colons\r\n").unwrap(),
            (&b"\r\n"[..], vec!["first", "last:with: colons"])
        );
    }

    #[test]
    fn raw_event_test() {
        assert_eq!(
            raw_event_parser(b":Angel!wings@irc.org PRIVMSG Wiz :Are you receiving this message ?\r\n").unwrap(),
            (&b""[..], RawEvent {
                prefix: Some(Prefix::User {
                    nick: "Angel",
                    username: Some("wings"),
                    host: "irc.org",
                }),
                command: "PRIVMSG",
                params: vec!["Wiz", "Are you receiving this message ?"],
            })
        );
        assert_eq!(
            raw_event_parser(b"PING\r\n").unwrap(),
            (&b""[..], RawEvent {
                prefix: None,
                command: "PING",
                params: vec![],
            })
        );
        assert_eq!(
            raw_event_parser(b":irc.example.com 001 test Welcome\r\n").unwrap(),
            (&b""[..], RawEvent {
                prefix: Some(Prefix::Server {
                    host: "irc.example.com"
                }),
                command: "001",
                params: vec!["test", "Welcome"],
            })
        );
    }

    #[test]
    fn event_test() {
        assert_eq!(
            parse_input(b":irc.example.com PING irc.example.com\r\n"),
            Event::Ping {
                server: "irc.example.com".to_string(),
                server2: None
            }
        );

        assert_eq!(
            parse_input(b"MADEUP command\r\n"),
            Event::Unknown
        );
    }
}
