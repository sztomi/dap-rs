use async_trait::async_trait;
use bytes::BytesMut;
use std::io::Error as IoError;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait LineReader {
    /// Read a single line and return it (terminator included)
    async fn read_line(&mut self) -> Result<String, IoError>;

    /// Read exactly `n` bytes and append them into `buffer`
    async fn read_n_bytes(&mut self, buffer: &mut BytesMut, n: usize) -> Result<usize, IoError>;
}

pub struct FileLineReader {
    pub file: File,
}

impl FileLineReader {
    pub async fn new(filepath: &str) -> Self {
        let file = File::open(filepath)
            .await
            .expect("failed to open input file {filepath}");
        FileLineReader { file }
    }
}

#[async_trait]
impl LineReader for FileLineReader {
    /// read an additional `n` bytes from the file and append them to `buffer`
    async fn read_n_bytes(&mut self, buffer: &mut BytesMut, n: usize) -> Result<usize, IoError> {
        let mut buf = BytesMut::with_capacity(n);
        self.file.read_exact(&mut buf).await?;
        buffer.extend_from_slice(&buf);
        Ok(n)
    }

    async fn read_line(&mut self) -> Result<String, IoError> {
        let mut buffer = BytesMut::with_capacity(128);
        loop {
            self.read_n_bytes(&mut buffer, 1).await?;
            // Check for LF `0x10`
            if *buffer.last().unwrap() as char == '\n' {
                // we have a complete line
                let line = String::from_utf8_lossy(&buffer).to_string();
                return Ok(line);
            }
        }
    }
}
