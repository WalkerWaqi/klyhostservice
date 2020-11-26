use std::io;
use std::path::Path;
use tokio::net::UnixStream;

#[allow(dead_code)]
struct UnixSocket {
    path: String,
    stream: UnixStream,
}

#[allow(dead_code)]
impl UnixSocket {
    async fn new(path: &str) -> io::Result<UnixSocket> {
        Ok(UnixSocket {
            stream: UnixStream::connect(Path::new(path)).await?,
            path: path.to_string(),
        })
    }
}
