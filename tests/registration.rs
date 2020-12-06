extern crate actor_ws;
use actor_ws::http::io::*;
use actor_ws::testing_tools::*;
type RegErrorOutput = ErrRepsonse<ResponseFeildError>;

#[cfg(test)]
mod testing_resgistration {
    use super::*;
    #[actix_rt::test]
    // testing if the sender resp and mock database retrieval is the same result
    // sender resp : user Uuid
    // retrieval from mock : emit list<user Uuid> -> select one
    async fn registering_test() {
        let mut srv = build_websocket_mock();
        let socket = srv.ws().await;
        let mut users_buff = String::new();
        let mut sender_res = String::new();
        match socket {
            Ok(mut ws) => {
                let sender = ws
                    .send(awc::ws::Message::Text("reg:register/ababa".to_string()))
                    .await;
                if let Err(e) = sender {
                    panic!("error during sending {}", e);
                } else {
                    match ws.next().await {
                        Some(resp) => {
                            if let Ok(awc::ws::Frame::Text(res)) = resp {
                                users_buff.push_str(
                                    &String::from_utf8(res.to_vec())
                                        .expect("error while parsing bytes!"),
                                )
                            }
                        }
                        None => {}
                    }
                }

                match ws.send(awc::ws::Message::Text("users".to_string())).await {
                    Ok(()) => match ws.next().await {
                        Some(resp) => match resp {
                            Ok(result) => {
                                if let awc::ws::Frame::Text(txt) = result {
                                    sender_res.push_str(
                                        &String::from_utf8(txt.to_vec())
                                            .expect("Sending result error. Bad parsing!"),
                                    )
                                } else {
                                    panic!("Unexpected frame!")
                                }
                            }
                            Err(e) => {
                                panic!("Protocol Violation caught!\ndetails:{}", e);
                            }
                        },
                        None => {
                            println!("no response!");
                        }
                    },
                    Err(e) => {
                        panic!("ws error {:#?}", e);
                    }
                }
            }
            Err(s) => {
                panic!("Client failure : {:#?}", s);
            }
        }
        let reg_output = serde_json::from_str::<RegistrationOutput>(users_buff.as_str())
            .expect("not expected result");

        assert_eq!(reg_output.user_id.to_string(), sender_res);
    }

    fn build_err_test(expect: [&'static str; 2]) -> Box<dyn Fn(RegErrorOutput)> {
        let main = expect[0];
        let sub = expect[1];
        let test = move |input: RegErrorOutput| {
            assert_eq!(input.error_type, main);
            assert_eq!(input.sub_type, sub);
        };
        return Box::new(test);
    }

    async fn failed_constraints<F>(failing_username: String, assertion: F)
    where
        F: Fn(RegErrorOutput),
    {
        let mut mock_srv = build_websocket_mock();
        let mut socket = mock_srv.ws().await.expect("bad socket init!");

        socket
            .send(awc::ws::Message::Text(format!(
                "reg:register/{}",
                failing_username
            )))
            .await
            .expect("error while sending message");

        let resp = socket.next().await;
        match resp {
            Some(res) => {
                if let Ok(awc::ws::Frame::Text(feedback)) = res {
                    println!("{:#?}", feedback);
                    let json_err_res =
                        String::from_utf8(feedback.to_vec()).expect("bad parsing error!");
                    let val = serde_json::from_str::<ErrRepsonse<ResponseFeildError>>(
                        json_err_res.as_str(),
                    )
                    .expect("Unencountered response!");
                    assertion(val);
                }
            }
            None => {
                dbg!("Empty response from the server!");
            }
        }
    }

    #[actix_rt::test]
    async fn failed_test() {
        let bad_formatted_text = "apple [ & */";
        let empty = "";
        let exceed_len = "aplp".repeat(100);
        let constraints = stream::iter(vec![
            (BADFORMAT, bad_formatted_text),
            (EMPTY, empty),
            (LENGTHLIMIT, &exceed_len),
        ]);

        constraints
            .for_each(|(con, fail)| async move {
                println!("{}", fail);
                let expectation = [REGISTRATION_ERR, con];
                let bad_form = build_err_test(expectation);
                failed_constraints(fail.to_string(), bad_form).await
            })
            .await;
    }
}

// just running stream iter
mod stupid_expirement {
    use super::*;

    async fn fake_future_test(input: i32) {
        println!("{}", input);
    }
    #[actix_rt::test]
    async fn stream_run() {
        let stream = stream::iter(vec![68, 290]);
        stream
            .for_each(|e| async move { fake_future_test(e).await })
            .await
    }
}
