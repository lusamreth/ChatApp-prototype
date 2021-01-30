use actor_ws::testing_tools::*;

#[cfg(test)]
mod testing_signal {
    use super::*;

    #[actix_rt::test]
    async fn connect() {
        let mut srv = build_websocket_mock();
        let mut socket = srv.ws().await.expect("failed to init socket!");

        socket
            .send(awc::ws::Message::Text("reg:register/apple_can".to_string()))
            .await
            .expect("bad sending!");

        match socket.next().await {
            Some(resp_res) => {
                let res = resp_res.expect("Protocol Error");
                if let awc::ws::Frame::Text(_) = res {
                    socket
                        .send(awc::ws::Message::Text(format!(
                            "signal:connect/{}",
                            uuid::Uuid::new_v4()
                        )))
                        .await
                        .expect("bad sending!");
                    let resa = socket.next().await.unwrap().expect("protocol Error");
                    if let awc::ws::Frame::Text(bytes) = resa {
                        let result = String::from_utf8(bytes.to_vec()).expect("Bad chunks!");
                        println!("this is result {:#?}", result);
                    }
                }
            }
            None => {
                panic!("No response from the server!");
            }
        }
    }
}
