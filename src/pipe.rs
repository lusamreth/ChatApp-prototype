use super::backend::{self, ClientMessage};
use crate::domain::{self, Registration};
use crate::http::io::*;
use actix::*;
use actix_web_actors::ws;
use domain::utility::build_extract_backlash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Clone)]
pub struct SimpleSocket {
    heartbeat: Instant,
    socket_id: Uuid, // == client
    room: Uuid,
    pub user: String,
    addr: Arc<Addr<backend::Server>>,
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

impl SimpleSocket {
    pub fn new(srv: Arc<Addr<backend::Server>>, room: Uuid, cl_id: Uuid) -> Self {
        let skid = cl_id;
        SimpleSocket {
            room,
            heartbeat: Instant::now(),
            addr: srv.clone(),
            socket_id: skid,
            user: String::new(),
        }
    }

    pub fn inject_soc_id(&mut self, new_id: Uuid) {
        self.socket_id = new_id;
    }

    fn adjust_hb(&self, ws_ctx: &mut ws::WebsocketContext<Self>) {
        ws_ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                // notify chat server
                // stop actor
                ctx.stop();
                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

    fn max_length_resp(
        &self,
        exceeded_len: usize,
        field: &str,
        ws_ctx: &mut ws::WebsocketContext<Self>,
    ) {
        let err = ErrResponse {
            error_type: REGISTRATION_ERR.to_string(),
            sub_type: LENGTHLIMIT.to_string(),
            instance: NA.to_string(),
            details: ResponseFeildError {
                field: "username".to_string(),
                error: format!("Field {} exceed length ; length : {}", field, exceeded_len),
            },
        };
        ws_ctx.text(serde_json::to_string(&err).expect("bad parsing json"));
    }

    fn register(
        &mut self,
        ws_ctx: &mut ws::WebsocketContext<Self>,
        input: &str,
        // server_act: Arc<Addr<backend::Server>>,
    ) {
        let scanner = build_extract_backlash("register", 16);
        match scanner(input) {
            Some(text_input) => {
                self.user = text_input.clone();
                let inputs = text_input
                    .split("/")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                if inputs.len() != 2 {
                    ws_ctx.text("parameters is not sufficient or bad format!");
                    return;
                }
                let _ = self
                    .addr
                    .send(Registration {
                        username: inputs.get(0).unwrap().clone(),
                        password: inputs.get(1).unwrap().clone(), //pwd:
                    })
                    .into_actor(self)
                    .then(move |res, act, ws_ctx| {
                        match res {
                            Ok(reg_res) => match reg_res.cl_id {
                                Some(id) => {
                                    println!("id created {:#?}", id);
                                    act.user = text_input;
                                    let init = std::mem::replace(&mut act.socket_id, id.clone());
                                    act.inject_soc_id(id);
                                    println!("initial id {:#?}", init);
                                    println!("socket id created {:#?}", act.socket_id);
                                    ws_ctx.text(reg_res.to_json());
                                }
                                None => {
                                    let err_json = reg_res.map_err();
                                    ws_ctx.text(err_json)
                                }
                            },
                            _ => {
                                ws_ctx.text("Something goes off!");
                                ws_ctx.stop()
                            }
                        }
                        fut::ready(())
                    })
                    .wait(ws_ctx);
            }
            None => {
                self.max_length_resp(input.len(), "username", ws_ctx);
            }
        }
    } //
    fn create_room(&mut self, ws_ctx: &mut ws::WebsocketContext<Self>, instruction: &str) {
        let extractor = build_extract_backlash("create_room", 32);
        match extractor(instruction) {
            Some(param) => {
                println!("this is socket id {}", self.socket_id);
                self.addr
                    .send(domain::CreateRoom {
                        name: param.to_string(),
                        creator: self.socket_id,
                        capacity: None,
                    })
                    .into_actor(self)
                    .then(|res, act, ws_ctx| {
                        match res {
                            Ok(output) => match output.handle {
                                Some(id) => {
                                    println!("sokc cret id {}", act.socket_id);
                                    act.room = id;
                                    let success = output.to_json();
                                    ws_ctx.text(success);
                                }
                                None => {
                                    let error = output.map_err();
                                    ws_ctx.text(error);
                                }
                            },
                            _ => ws_ctx.stop(),
                        }
                        fut::ready(())
                    })
                    .wait(ws_ctx);
            }
            None => {
                self.max_length_resp(instruction.len(), "room_name", ws_ctx);
            }
        }
    }
    // incomming message ! : signal:{token}/{user_id}
    fn process_signal(&mut self, ws_ctx: &mut ws::WebsocketContext<Self>, input: &str) {
        let res_token: Option<domain::SignalCode>;
        let halves = input.split("/").collect::<Vec<&str>>();
        let default = String::new();
        let mut param = String::new();
        match halves
            .first()
            .unwrap_or(&default.as_str())
            .to_lowercase()
            .as_str()
        {
            "connect" => {
                if let Some(p) = halves.get(1) {
                    res_token = Some(domain::SignalCode::Connect);
                    param.push_str(p);
                } else {
                    res_token = None;
                }
            }
            "disconnect" => res_token = Some(domain::SignalCode::Disconnect),
            "pending" => res_token = Some(domain::SignalCode::Pending),
            _ => res_token = None,
        }
        // need id?
        //    let extract = build_extract_backlash("connect", 32);
        let addr = ws_ctx.address().recipient();

        match Uuid::parse_str(&param) {
            Ok(id) => match res_token {
                Some(token) => {
                    self.addr
                        .send(domain::SignalInput {
                            id: self.socket_id,
                            room_id: id,
                            code: token,
                            addr,
                        })
                        .into_actor(self)
                        .then(move |res, act, ctx| {
                            match res {
                                Ok(output) => {
                                    act.room = id;
                                    let resp = match output.status {
                                        domain::ConnectionStatus::Aborted(_) => output.map_err(),
                                        _ => output.to_json(),
                                    };
                                    ctx.text(resp);
                                }
                                Err(_) => ctx.stop(),
                            }
                            fut::ready(())
                        })
                        .wait(ws_ctx);
                }
                None => ws_ctx.text("Unknown Token!"),
            },
            Err(_) => ws_ctx.text("Invalid room id!"),
        }
    }

    fn process_join(&mut self, ws_ctx: &mut ws::WebsocketContext<Self>, input: &str) {
        let extract = build_extract_backlash("join_room", 100);
        println!("P{}", input);
        match extract(input) {
            Some(room_id) => match Uuid::parse_str(&room_id) {
                Ok(id) => {
                    println!("join id {}", self.socket_id);
                    self.addr
                        .send(domain::JoinInput {
                            target_id: id,
                            client_id: self.socket_id,
                            addr: ws_ctx.address().recipient(),
                        })
                        .into_actor(self)
                        .then(|res, _, ctx| {
                            match res {
                                Ok(item) => {
                                    let res = match item {
                                        domain::JoinOutput::Success => item.to_json(),
                                        _ => item.map_err(),
                                    };
                                    ctx.text(res);
                                }
                                _ => ctx.stop(),
                            }
                            fut::ready(())
                        })
                        .wait(ws_ctx);
                }
                Err(_) => ws_ctx.text("Incorrect Id!"),
            },
            None => ws_ctx.text("Unexpected Input!"),
        }
    }
}

impl Actor for SimpleSocket {
    fn started(&mut self, ctx: &mut Self::Context) {
        self.adjust_hb(ctx);
        //self.addr.send()
    }
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        //println!("Stopping addrs {:#?}",ctx.address());
        Running::Stop
    }
    type Context = ws::WebsocketContext<Self>;
}

type Websocket = Result<ws::Message, ws::ProtocolError>;
impl StreamHandler<Websocket> for SimpleSocket {
    fn handle(&mut self, msg: Websocket, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(ms) => ms,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        match msg {
            ws::Message::Binary(_) => {
                ctx.text("Unknown Binary code");
            }
            ws::Message::Close(res) => {
                ctx.close(res);
            }
            ws::Message::Ping(ping) => {
                // reset the heartbeat
                self.heartbeat = Instant::now();
                ctx.pong(&ping);
            }
            ws::Message::Pong(_) => {
                // reset the heartbeat
                self.heartbeat = Instant::now();
            }
            ws::Message::Text(text) => {
                // request via paramter!

                // request to server with operation instruction
                let opcode = text.trim().split(":").collect::<Vec<&str>>();
                if opcode.len() == 1 {
                    let code = opcode.first().map(|s| *s);
                    println!("code {:#?}", code);
                    match code.unwrap_or(String::new().as_str()) {
                        "users" => {
                            self.addr
                                .send(domain::ListUser {})
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(result) => {
                                            if result.len() == 0 {
                                                ctx.text("empty user base!")
                                            } else {
                                                for (id, _) in result.iter() {
                                                    ctx.text(String::from(id.to_string()));
                                                }
                                            }
                                        }
                                        _ => ctx.stop(),
                                    }
                                    return fut::ready(());
                                })
                                .wait(ctx);
                            //
                        }
                        "rooms" => {
                            self.addr
                                .send(domain::RoomRetrieval {})
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(retrieval_res) => match retrieval_res {
                                            Ok(rooms) => {
                                                rooms.iter().for_each(|(id, room)| {
                                                    let val = serde_json::to_string(&Rooms {
                                                        id: *id,
                                                        room_name: room.name.clone(),
                                                        capacity: room.capacity.unwrap_or(999999),
                                                        participants: room.participants,
                                                    })
                                                    .expect("bad parsing ");
                                                    ctx.text(val);
                                                });
                                            }
                                            Err(fail) => ctx.text(fail.to_string()),
                                        },
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                        }
                        _ => ctx.text("unexpected opcode"),
                    }
                } else if opcode.len() == 2 {
                    let code = *opcode.first().unwrap();
                    let instruction = *opcode.get(1).unwrap();
                    match code {
                        "room" => {
                            self.create_room(ctx, instruction);
                        }
                        "join" => {
                            self.process_join(ctx, instruction);
                        }
                        "reg" => {
                            self.register(ctx, instruction);
                        }
                        "signal" => {
                            self.process_signal(ctx, instruction);
                        }
                        "text" => {
                            //self.addr.send(msg);
                            let message = instruction.trim().to_string();
                            self.addr.do_send(ClientMessage {
                                cl_id: self.socket_id,
                                message,
                                room_id: self.room,
                            })
                        }
                        _ => ctx.text("bad opcode"),
                    }
                }
                // self.clone().register(ctx, &text, self.addr.clone());
            }
            ws::Message::Continuation(_) => ctx.text("Unsupported!"),
            ws::Message::Nop => ctx.text("Unsupported!"),
        }
    }
}

// handle message payload!
impl Handler<domain::Payload> for SimpleSocket {
    type Result = ();

    fn handle(&mut self, msg: domain::Payload, ctx: &mut Self::Context) {
        let msg = MessagePayload {
            message: msg.message,
            date: msg.date,
        };
        let str_msg = serde_json::to_string(&msg).expect("failed to serialize message!");
        ctx.text(str_msg);
    }
}
