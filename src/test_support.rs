use zbus::{connection::Builder, object_server::Interface, Connection};

pub async fn peer_pair(service: impl Interface, path: &'static str) -> (Connection, Connection) {
    let (server, client) = std::os::unix::net::UnixStream::pair().unwrap();
    let server = Builder::unix_stream(server)
        .server(zbus::Guid::generate())
        .p2p()
        .serve_at(path, service)
        .build();
    let client = Builder::unix_stream(client).p2p().build();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        tokio::try_join!(server, client).unwrap()
    })
    .await
    .unwrap()
}
