extern crate actor_ws;
use actor_ws::testing_tools::*;
use std::sync::{Arc, Mutex};

const INTENDED_MESSAGE: &'static str = "NGe is very depressing!";

#[cfg(test)]
mod text_testing {
    use super::*;

    #[actix_rt::test]
    async fn send_mock_text() {
        let mut srv = build_websocket_mock();

        let instructions = vec![
            "reg:register/ababab".to_string(),
            "signal:connect".to_string(),
            "room:create_room/apacaba".to_string(),
            format!("text:{}", INTENDED_MESSAGE),
        ];
        let socket = srv.ws().await.expect("failed while init socket!");
        let sc = Arc::new(Mutex::new(socket));
        stream::iter(instructions)
            .for_each(|ex| async {
                println!("{:#?}", ex);
                let s = sc.clone();
                let mut sender = s.lock().expect("Poision");
                sender
                    .send(awc::ws::Message::Text(ex))
                    .await
                    .expect("failed while sending the message");

                // let frame = sc.lock().expect("bruh").next().await.expect("cannot recieve value!");
                // let frame = sender.next().await.unwrap();
                // if let awc::ws::Frame::Text(chunk) = frame.unwrap() {
                //     let msg = String::from_utf8(chunk.to_vec()).expect("bad message!");
                //     println!("msg {}", msg);
                // }
            })
            .await;
        let mut locked = sc.lock().unwrap();
        let mut index: i32 = 0;

        while let Some(item) = locked.next().await {
            if let awc::ws::Frame::Text(chunk) = item.unwrap() {
                let msg = String::from_utf8(chunk.to_vec()).expect("bad message!");
                let payload = serde_json::from_str::<http::io::MessagePayload>(&msg).unwrap();
                if index == 4 {
                    assert_eq!(INTENDED_MESSAGE.to_string(), payload.message);
                }
                index += 1;
                println!("msg {}", msg);
            }
        }
    }
}
