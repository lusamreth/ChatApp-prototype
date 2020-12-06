extern crate actor_ws;
use actix_codec::{AsyncRead, AsyncWrite, Framed};
use actor_ws::http::io::*;
use actor_ws::testing_tools::*;
use awc::ws::Codec;
use std::sync::{Mutex,Arc};
trait BoogieMan {}
// type d = Box<dyn AsyncWrite + AsyncWrite>;
pub struct Socket<T: AsyncWrite + AsyncRead>(Framed<T, Codec>);

#[cfg(test)]
mod room_test {

    use super::*;

    pub fn from_json<T>(frame: awc::ws::Frame, converter: impl Fn(String) -> T) -> T {
        if let awc::ws::Frame::Text(chunk) = frame {
            let str_val = String::from_utf8(chunk.to_vec()).expect("bad chunk!");
            converter(str_val)
        } else {
            panic!("Unexpected frame");
        }
    }

    async fn create_room_test<T: AsyncRead + AsyncWrite + Unpin>(
        socket: &mut Socket<T>,
        name: &str,
    ) -> uuid::Uuid {
        socket
            .0
            .send(awc::ws::Message::Text(format!("room:create_room/{}", name)))
            .await
            .expect("error while sending message");
        let resp: Option<RoomCreationOutput>;
        let creation =
            |str_val: String| serde_json::from_str::<RoomCreationOutput>(&str_val).unwrap();
        match socket.0.next().await {
            Some(result) => match result {
                Ok(frame) => {
                    resp = Some(from_json(frame, creation));
                }
                Err(prot) => {
                    dbg!("protocol error!");
                    panic!(prot);
                }
            },
            None => {
                panic!("Server don't response to message");
            }
        }
        return resp.unwrap().room_id;
    }

    async fn retrieve_room<T: AsyncRead + AsyncWrite + Unpin>(socket: &mut Socket<T>) -> Rooms {
        let retrieval = |str_val: String| serde_json::from_str::<Rooms>(&str_val).unwrap();
        socket
            .0
            .send(awc::ws::Message::Text("rooms".to_string()))
            .await
            .expect("error while sending message");
        let re = socket.0.next().await.unwrap();
        let ret = from_json(re.unwrap(), retrieval);
        return ret;
    }

    async fn join_room<T: AsyncRead + AsyncWrite + Unpin>(
        socket: &mut Socket<T>,
        room_id: String,
    ) -> JoinResp {
        let _ = socket
            .0
            .send(awc::ws::Message::Text(format!(
                "join:join_room/{}",
                room_id
            )))
            .await;
        match socket.0.next().await {
            Some(item) => {
                if let awc::ws::Frame::Text(b) = item.unwrap() {
                    let txt = &String::from_utf8(b.to_vec()).unwrap();
                    println!("txt {}", txt);
                    serde_json::from_str::<JoinResp>(txt).unwrap()
                } else {
                    panic!("unexpected frame!")
                }
            }
            None => panic!("stop!"),
        }
    }

    async fn register<T: AsyncRead + AsyncWrite + Unpin>(
        socket: &mut Socket<T>,
        name: String,
    ) -> RegistrationOutput {
        let _ = socket
            .0
            .send(awc::ws::Message::Text(format!("reg:register/{}", name)))
            .await;
        let converter = |txt: String| {
            println!("reg txt {}", txt);
            serde_json::from_str::<RegistrationOutput>(&txt).expect("bad register parsing!")
        };
        let frame = socket.0.next().await.unwrap().expect("protocol error!");
        return from_json(frame, converter);
    }

    async fn connect<T: AsyncRead + AsyncWrite + Unpin>(
        socket: &mut Socket<T>,
        room_id: String,
    ) -> SignalPayload {
        let _ = socket
            .0
            .send(awc::ws::Message::Text(format!(
                "signal:connect/{}",
                room_id
            )))
            .await;
        let converter = |txt: String| {
            println!("txt {}", txt);
            serde_json::from_str::<SignalPayload>(&txt).expect("bad parsing!")
        };
        let frame = socket.0.next().await.unwrap().expect("protocol error!");
        return from_json(frame, converter);
    }

    #[actix_rt::test]
    async fn run_room_test() {
        let mut srv = build_websocket_mock();
        let mut socket = Socket(srv.ws().await.unwrap());

        //
        let testing_room= create_room_test(&mut socket, "socks").await;
        let _ = register(&mut socket, "Superman!".to_string()).await;
        let join_output = join_room(&mut socket, testing_room.to_string()).await;
        let retrieval = retrieve_room(&mut socket).await;
        let conn = connect(&mut socket, testing_room.to_string()).await;

        assert_eq!(conn.status,"Connected");
        assert_eq!(retrieval.id,testing_room);
        assert_eq!(retrieval.room_name,"socks");
        assert_eq!(join_output.status.to_lowercase(),"success");

        // after passing the authentication process!
        // start sending message
        const UNITS_2000:[i32;2000] = [0;2000];
        let sync_socket= Arc::new(Mutex::new(socket.0));
        let msg = "Mr apple pie juice 10280!";
        stream::iter(&UNITS_2000).for_each(|_| async {
           let _ = sync_socket.lock().unwrap().send(awc::ws::Message::Text(format!("text:{}",msg))).await;
        }).await;
        let mut locked = sync_socket.lock().unwrap();
        while let Some(item) = locked.next().await {
            if let awc::ws::Frame::Text(chunk) = item.unwrap() {
                let msg = String::from_utf8(chunk.to_vec()).expect("bad message!");
                println!("msg {}", msg);
                assert_eq!(msg,msg);
            }
        }
    }
}
