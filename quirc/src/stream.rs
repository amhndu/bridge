use crate::utils;
use std::io::{Read, Result, Write};

pub trait Stream: Read + Write {}
impl<T> Stream for T where T: Read + Write {}

struct StreamParser<'a, T: Read> {
    stream: &'a mut T,
    buffer: Vec<u8>,
}

struct StreamWriter<'a, T: Write> {
    stream: &'a mut T,
}

const CRLF: &[u8] = b"\r\n";

impl<'a, T: Read> StreamParser<'a, T> {
    fn new(stream: &'a mut T) -> Self {
        StreamParser {
            stream,
            buffer: Vec::with_capacity(512),
        }
    }

    fn read_line(&mut self) -> Option<Vec<u8>> {
        let mut line_end = utils::find_subsequence(&self.buffer[..], CRLF);
        let mut read_count = 0;
        let mut buf_end = self.buffer.len();
        self.buffer.resize(512, 0);
        while let None = line_end {
            read_count = buf_end;

            match self.stream.read(&mut self.buffer[read_count..]) {
                Ok(0) if self.buffer.len() == 0 => return Some(Vec::new()),
                Ok(size) => {
                    buf_end += size;
                    line_end = utils::find_subsequence(&self.buffer[read_count..buf_end], CRLF);
                }
                _ => return None,
            }
        }

        let line_end = line_end.unwrap();
        self.buffer.truncate(buf_end); // buffer was resized to 512, truncate it till data
        let mut split = self.buffer.split_off(read_count + line_end + CRLF.len()); // split after the CRLF
        std::mem::swap(&mut split, &mut self.buffer); // swap to get the leftover on buffer

        Some(split)
    }
}

impl<'a, T: Write> StreamWriter<'a, T> {
    fn new(stream: &'a mut T) -> Self {
        StreamWriter { stream }
    }

    fn write_line(&mut self, data: &Vec<u8>) -> bool {
        self.stream.write_all(&data[..]).is_ok() && self.stream.write_all(CRLF).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::default::Default;

    #[derive(Default)]
    struct ChunkedTestStream {
        read_buf: Vec<u8>,
        bytes_read: usize,
        chunk_size: usize,
        write_buf: Vec<u8>,
    }

    impl Read for ChunkedTestStream {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let size = self
                .read_buf
                .len()
                .checked_sub(self.bytes_read)
                .unwrap_or(std::cmp::min(buf.len(), self.chunk_size));
            buf[..size].clone_from_slice(&self.read_buf[self.bytes_read..self.bytes_read + size]);
            self.bytes_read += size;
            Ok(size)
        }
    }

    impl Write for ChunkedTestStream {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let size = std::cmp::min(buf.len(), self.chunk_size);
            self.write_buf.extend(&buf[..size]);
            Ok(size)
        }

        fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
            Ok(())
        }
    }

    #[test]
    fn read_line_test() {
        let mut stream = ChunkedTestStream {
            read_buf: b"line-one\r\nline-two\r\nline-three\r\n".to_vec(),
            chunk_size: 5,
            ..Default::default()
        };

        let mut parser = StreamParser::new(&mut stream);
        assert_eq!(parser.read_line().unwrap(), b"line-one\r\n".to_vec());
        assert_eq!(parser.read_line().unwrap(), b"line-two\r\n".to_vec());
        assert_eq!(parser.read_line().unwrap(), b"line-three\r\n".to_vec());
    }

    #[test]
    fn write_line_test() {
        let mut stream = ChunkedTestStream {
            chunk_size: 5,
            ..Default::default()
        };

        let mut writer = StreamWriter::new(&mut stream);
        assert!(writer.write_line(&b"line-one".to_vec()));
        assert!(writer.write_line(&b"line-two".to_vec()));
        assert!(writer.write_line(&b"line-three".to_vec()));
        assert_eq!(&stream.write_buf, b"line-one\r\nline-two\r\nline-three\r\n");
    }
}
