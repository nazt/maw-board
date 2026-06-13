use anyhow::{Context, Result};
use sshx::{controller::Controller, encrypt::Encrypt, runner::Runner};
use sshx_core::{
    proto::{server_update::ServerMessage, NewShell, TerminalInput},
    Sid, Uid,
};
use sshx_server::web::protocol::{WsClient, WsWinsize};
use tokio::time::{self, Duration};

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_handshake() -> Result<()> {
    let server = TestServer::new().await;
    let controller = Controller::new(&server.endpoint(), "", Runner::Echo, false).await?;
    controller.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_command() -> Result<()> {
    let server = TestServer::new().await;
    let runner = Runner::Shell("/bin/bash".into());
    let mut controller = Controller::new(&server.endpoint(), "", runner, false).await?;

    let session = server
        .state()
        .lookup(controller.name())
        .context("couldn't find session in server state")?;

    let updates = session.update_tx();
    let new_shell = NewShell { id: 1, x: 0, y: 0 };
    updates.send(ServerMessage::CreateShell(new_shell)).await?;

    let key = controller.encryption_key();
    let encrypt = Encrypt::new(key);
    let offset = 4242;
    let data = TerminalInput {
        id: 1,
        data: encrypt.segment(0x200000000, offset, b"ls\r\n").into(),
        offset,
    };
    updates.send(ServerMessage::Input(data)).await?;

    tokio::select! {
        _ = controller.run() => (),
        _ = time::sleep(Duration::from_millis(1000)) => (),
    };
    controller.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_ws_missing() -> Result<()> {
    let server = TestServer::new().await;

    let bad_endpoint = format!("ws://{}/not/an/endpoint", server.local_addr());
    assert!(ClientSocket::connect(&bad_endpoint, "", None)
        .await
        .is_err());

    let mut s = ClientSocket::connect(&server.ws_endpoint("foobar"), "", None).await?;
    s.expect_close(4404).await;

    Ok(())
}

#[tokio::test]
async fn test_ws_basic() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), "", Runner::Echo, false).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let mut s = ClientSocket::connect(&server.ws_endpoint(&name), &key, None).await?;
    s.flush().await;
    assert_eq!(s.user_id, Uid(1));

    s.send(WsClient::Create(0, 0)).await;
    s.flush().await;
    assert_eq!(s.shells.len(), 1);
    assert!(s.shells.contains_key(&Sid(1)));

    s.send(WsClient::Subscribe(Sid(1), 0)).await;
    assert_eq!(s.read(Sid(1)), "");

    s.send_input(Sid(1), b"hello!").await;
    s.flush().await;
    assert_eq!(s.read(Sid(1)), "hello!");

    s.send_input(Sid(1), b" 123").await;
    s.flush().await;
    assert_eq!(s.read(Sid(1)), "hello! 123");

    Ok(())
}

#[tokio::test]
async fn test_ws_resize() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), "", Runner::Echo, false).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let mut s = ClientSocket::connect(&server.ws_endpoint(&name), &key, None).await?;

    s.send(WsClient::Move(Sid(1), None)).await; // error: does not exist yet!
    s.flush().await;
    assert_eq!(s.errors.len(), 1);

    s.send(WsClient::Create(0, 0)).await;
    s.flush().await;
    assert_eq!(s.shells.len(), 1);
    assert_eq!(*s.shells.get(&Sid(1)).unwrap(), WsWinsize::default());

    let new_size = WsWinsize {
        x: 42,
        y: 105,
        rows: 200,
        cols: 20,
    };
    s.send(WsClient::Move(Sid(1), Some(new_size))).await;
    s.send(WsClient::Move(Sid(2), Some(new_size))).await; // error: does not exist
    s.flush().await;
    assert_eq!(s.shells.len(), 1);
    assert_eq!(*s.shells.get(&Sid(1)).unwrap(), new_size);
    assert_eq!(s.errors.len(), 2);

    s.send(WsClient::Close(Sid(1))).await;
    s.flush().await;
    assert_eq!(s.shells.len(), 0);

    s.send(WsClient::Move(Sid(1), None)).await; // error: shell was closed
    s.flush().await;
    assert_eq!(s.errors.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_users_join() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), "", Runner::Echo, false).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut s1 = ClientSocket::connect(&endpoint, &key, None).await?;
    s1.flush().await;
    assert_eq!(s1.users.len(), 1);

    let mut s2 = ClientSocket::connect(&endpoint, &key, None).await?;
    s2.flush().await;
    assert_eq!(s2.users.len(), 2);

    drop(s2);
    let mut s3 = ClientSocket::connect(&endpoint, &key, None).await?;
    s3.flush().await;
    assert_eq!(s3.users.len(), 2);

    s1.flush().await;
    assert_eq!(s1.users.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_users_metadata() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), "", Runner::Echo, false).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut s = ClientSocket::connect(&endpoint, &key, None).await?;
    s.flush().await;
    assert_eq!(s.users.len(), 1);
    assert_eq!(s.users.get(&s.user_id).unwrap().cursor, None);

    s.send(WsClient::SetName("mr. foo".into())).await;
    s.send(WsClient::SetCursor(Some((40, 524)))).await;
    s.flush().await;
    let user = s.users.get(&s.user_id).unwrap();
    assert_eq!(user.name, "mr. foo");
    assert_eq!(user.cursor, Some((40, 524)));

    Ok(())
}

#[tokio::test]
async fn test_chat_messages() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), "", Runner::Echo, false).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut s1 = ClientSocket::connect(&endpoint, &key, None).await?;
    let mut s2 = ClientSocket::connect(&endpoint, &key, None).await?;

    s1.send(WsClient::SetName("billy".into())).await;
    s1.send(WsClient::Chat("hello there!".into())).await;
    s1.flush().await;

    s2.flush().await;
    assert_eq!(s2.messages.len(), 1);
    assert_eq!(
        s2.messages[0],
        (s1.user_id, "billy".into(), "hello there!".into())
    );

    let mut s3 = ClientSocket::connect(&endpoint, &key, None).await?;
    s3.flush().await;
    assert_eq!(s1.messages.len(), 1);
    assert_eq!(s3.messages.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_read_write_permissions() -> Result<()> {
    let server = TestServer::new().await;

    // create controller with read-only mode enabled
    let mut controller = Controller::new(&server.endpoint(), "", Runner::Echo, true).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    let write_url = controller
        .write_url()
        .expect("Should have write URL when enable_readers is true")
        .to_string();

    tokio::spawn(async move { controller.run().await });

    let write_password = write_url
        .split(',')
        .nth(1)
        .expect("Write URL should contain password");

    // connect with write access
    let mut writer =
        ClientSocket::connect(&server.ws_endpoint(&name), &key, Some(write_password)).await?;
    writer.flush().await;

    // test write permissions
    writer.send(WsClient::Create(0, 0)).await;
    writer.flush().await;
    assert_eq!(
        writer.shells.len(),
        1,
        "Writer should be able to create a shell"
    );
    assert!(writer.errors.is_empty(), "Writer should not receive errors");

    // connect with read-only access
    let mut reader = ClientSocket::connect(&server.ws_endpoint(&name), &key, None).await?;
    reader.flush().await;

    // test read-only restrictions
    reader.send(WsClient::Create(0, 0)).await;
    reader.flush().await;
    assert!(
        !reader.errors.is_empty(),
        "Reader should receive an error when attempting to create shell"
    );
    assert_eq!(
        reader.shells.len(),
        1,
        "Reader should still see the existing shell"
    );

    Ok(())
}

#[tokio::test]
async fn test_files_api() -> Result<()> {
    let server = TestServer::new().await;
    let client = reqwest::Client::new();

    // 1. Test listing files
    let url_list = format!("http://{}/api/files?path=", server.local_addr());
    let resp = client.get(&url_list).send().await?;
    assert_eq!(resp.status(), http::StatusCode::OK);
    let text = resp.text().await?;
    assert!(text.contains("\"path\":"));
    assert!(text.contains("\"items\":"));

    // Create a temporary file inside /root/maw-workspace to test reading
    let temp_file_path = std::path::Path::new("/root/maw-workspace/test_hello.txt");
    tokio::fs::write(&temp_file_path, b"hello workspace file!").await?;

    // 2. Test reading the file
    let url_read = format!("http://{}/api/file?path=test_hello.txt", server.local_addr());
    let resp = client.get(&url_read).send().await?;
    assert_eq!(resp.status(), http::StatusCode::OK);
    assert_eq!(resp.headers().get(http::header::CONTENT_TYPE).unwrap(), "application/json");
    assert_eq!(resp.headers().get(http::header::CACHE_CONTROL).unwrap(), "no-cache");
    let json_text = resp.text().await?;
    assert!(json_text.contains("\"path\":\"test_hello.txt\""));
    assert!(json_text.contains("\"content\":\"hello workspace file!\""));

    // Clean up
    tokio::fs::remove_file(&temp_file_path).await.ok();

    // 3. Test reading nonexistent file
    let url_missing = format!("http://{}/api/file?path=does_not_exist.txt", server.local_addr());
    let resp = client.get(&url_missing).send().await?;
    assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);

    // 4. Test directory traversal attempt
    let url_traversal = format!("http://{}/api/file?path=../etc/passwd", server.local_addr());
    let resp = client.get(&url_traversal).send().await?;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

    // 5. Test dotfile rejection
    let dotfile_path = std::path::Path::new("/root/maw-workspace/.test_dotfile");
    tokio::fs::write(&dotfile_path, b"hidden content").await?;
    let url_dotfile = format!("http://{}/api/file?path=.test_dotfile", server.local_addr());
    let resp = client.get(&url_dotfile).send().await?;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    tokio::fs::remove_file(&dotfile_path).await.ok();

    // 6. Test directory rejection
    let temp_dir_path = std::path::Path::new("/root/maw-workspace/test_dir");
    tokio::fs::create_dir(&temp_dir_path).await?;
    let url_dir = format!("http://{}/api/file?path=test_dir", server.local_addr());
    let resp = client.get(&url_dir).send().await?;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    tokio::fs::remove_dir(&temp_dir_path).await.ok();

    // 7. Test binary file rejection (invalid UTF-8 bytes)
    let binary_path = std::path::Path::new("/root/maw-workspace/test_binary.bin");
    tokio::fs::write(&binary_path, b"hello \xff\xff world").await?;
    let url_binary = format!("http://{}/api/file?path=test_binary.bin", server.local_addr());
    let resp = client.get(&url_binary).send().await?;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    assert_eq!(resp.text().await?, "binary file");
    tokio::fs::remove_file(&binary_path).await.ok();

    // 8. Test file size > 1 MiB -> 413 Payload Too Large
    let large_path = std::path::Path::new("/root/maw-workspace/test_large.txt");
    let large_data = vec![b'a'; 1024 * 1024 + 10];
    tokio::fs::write(&large_path, &large_data).await?;
    let url_large = format!("http://{}/api/file?path=test_large.txt", server.local_addr());
    let resp = client.get(&url_large).send().await?;
    assert_eq!(resp.status(), http::StatusCode::PAYLOAD_TOO_LARGE);
    tokio::fs::remove_file(&large_path).await.ok();

    Ok(())
}

