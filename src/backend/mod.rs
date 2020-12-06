use crate::domain::*;
use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;
//use env_logger
mod handlers;
pub use super::domain::ClientMessage;
use super::domain::{Client, Payload, Room, User};
use std::sync::*;
type Clients = Arc<RwLock<HashMap<Uuid, Client>>>;
type Rooms = Arc<RwLock<HashMap<Uuid, Room>>>;

// way of storing messages
// Room : store hasmap of msgs
// [client_id : "message"] // hashmap

type RoomId = Uuid;
type UserId = Uuid;

#[derive(Debug)]
pub struct Server {
    server_id: Uuid,
    pub clients: Clients,
    rooms: Rooms,
}

impl Server {
    pub fn create() -> Self {
        let ent = Server {
            server_id: Uuid::new_v4(),
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        };
        return ent;
    }

    fn send_message(&self, room_id: Uuid, message: &str, client_id: Uuid) {
        println!("Sending message from server with id {}", self.server_id);
        let mut handle = self.rooms.write().unwrap();
        // println!("S{:#?}",self.clients);
        match handle.get_mut(&room_id) {
            Some(room) => {
                let mut payload_stack = Vec::with_capacity(1);
                room.client_ids.iter_mut().for_each(|cl| {
                    // println!("{}",cl);
                    if client_id == *cl {
                        let clients = self.clients.read().unwrap(); // we know it exist
                        match clients.get(&client_id) {
                            Some(client) => {
                                if let Some(ref addr) = client.address {
                                    let mut payload = Payload::new(message);
                                    payload_stack.push(payload.clone());
                                    //room.append_message(payload, client.client_id);
                                    let _ = addr.do_send(payload);
                                }
                            }
                            None => {}
                        }
                    } else {
                        // no conditional statement
                        //
                    }
                });
                if payload_stack.len() > 0 {
                    room.append_message(payload_stack.first().unwrap().clone(), client_id);
                    println!("payload stack {:#?}", payload_stack);
                }
            }
            None => {
                println!("No MESSAGE is being send");
            }
        };
    }

    pub fn create_room(
        &mut self,
        room_name: String,
        cap: Option<i32>,
        admin: Uuid,
    ) -> RoomCreation {
        // check against the clients list to see if user existed!
        match self.clients.read() {
            Ok(handle) => {
                if handle.get(&admin).is_some() {
                    let new_room = Room::create(room_name, cap, admin);
                    match new_room {
                        Ok(new_room) => match self.rooms.write() {
                            Ok(mut handle) => {
                                let id = new_room.room_id;
                                handle.insert(id.clone(), new_room);
                                return RoomCreation {
                                    status: RoomCreationStatus::CREATED,
                                    handle: Some(id),
                                };
                            }
                            Err(_) => {
                                let stat = RoomCreationStatus::ERROR(RoomError::INTERNALERROR(
                                    FailureReason::ACCESSWRITEERROR,
                                ));
                                return RoomCreation {
                                    status: stat,
                                    handle: None,
                                };
                            }
                        },
                        Err(error) => {
                            return RoomCreation {
                                status: RoomCreationStatus::ERROR(error),
                                handle: None,
                            }
                        }
                    }
                } else {
                    let status =
                        RoomCreationStatus::ERROR(RoomError::REFUSED(RoomRejection::UnknownUser));
                    return RoomCreation {
                        status,
                        handle: None,
                    };
                }
            }
            Err(err) => {
                let status = RoomCreationStatus::ERROR(RoomError::INTERNALERROR(
                    FailureReason::ACCESSWRITEERROR,
                ));
                return RoomCreation {
                    status,
                    handle: None,
                };
            }
        }
    }

    pub fn handle_register(&mut self, username: String) -> RegisterRes {
        let usn_field = String::from("username");
        if username.len() == 0 {
            return RegisterRes {
                status: RegistrationStatus::REFUSED(RefusedReason::EMPTY, usn_field),
                cl_id: None,
            };
        }
        if utility::sanitize_text(username.as_str()) {
            return RegisterRes {
                status: RegistrationStatus::REFUSED(RefusedReason::BADFORMAT, usn_field),
                cl_id: None,
            };
        }
        let new_id = Uuid::new_v4();
        let new_cl = Client {
            client_id: new_id.clone(),
            address: None,
            user: User::new(username),
        };
        match self.clients.read() {
            Ok(handle) => {
                if let Some(_) = handle.get(&new_id) {
                    return RegisterRes {
                        status: RegistrationStatus::FAILED(FailureReason::COLLISION),
                        cl_id: None,
                    };
                }
            }
            Err(e) => {
                dbg!("poision read handle", e);
                return RegisterRes {
                    status: RegistrationStatus::FAILED(FailureReason::ACCESSREADERROR),
                    cl_id: None,
                };
            }
        }
        // user name must not contain ${#,[,],?,*,/,\,','}
        // append new user
        println!("id col {}", new_id);
        match self.clients.write() {
            Ok(mut handle) => {
                handle.insert(new_id, new_cl);
            }
            Err(e) => {
                dbg!("poision write handle", e);
                return RegisterRes {
                    status: RegistrationStatus::FAILED(FailureReason::ACCESSWRITEERROR),
                    cl_id: None,
                };
            }
        }
        return RegisterRes {
            status: RegistrationStatus::CREATED,
            cl_id: Some(new_id),
        };
    }

    pub fn retrieve_rooms(&self) -> Result<HashMap<RoomId, Room>, FailureReason> {
        let mut result = HashMap::new();

        match self.rooms.read() {
            Ok(handle) => {
                handle.iter().for_each(|(id, room)| {
                    result.insert(*id, room.clone());
                });
                return Ok(result);
            }
            Err(_) => {
                println!("Poisioned handle!");
                Err(FailureReason::ACCESSREADERROR)
                // return result;
            }
        }
    }

    pub fn retrieve_users(&mut self) -> HashMap<UserId, User> {
        let mut result = HashMap::new();
        match self.clients.read() {
            Ok(cls) => {
                cls.iter().for_each(|(e, s)| {
                    let user = s.user.clone();
                    result.insert(e.clone(), user);
                });
            }
            Err(e) => {
                dbg!("posion read users", &e);
                return result;
            }
        }
        result
    }
}

impl Actor for Server {
    type Context = Context<Self>; // give it the prebuilt context
}

impl Handler<ClientMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Self::Context) -> Self::Result {
        self.send_message(msg.room_id, &msg.message, msg.cl_id);
    }
}
impl Handler<JoinInput> for Server {
    type Result = MessageResult<JoinInput>;

    fn handle(&mut self, msg: JoinInput, _: &mut Self::Context) -> Self::Result {
        let mut rooms = self.rooms.write().expect("Error while joinning");
        match rooms.get_mut(&msg.target_id) {
            Some(target) => {
                println!("reach room!");
                let mut cls = self.clients.write().unwrap();
                println!("ths is recieved id {:#?}", &msg.client_id);
                if let Some(client) = cls.get_mut(&msg.client_id) {
                    if target
                        .client_ids
                        .iter()
                        .find(|x| *x == &msg.client_id)
                        .is_some()
                    {
                        return MessageResult(JoinOutput::Failed(FailureReason::COLLISION));
                    } else {
                        match client.address {
                            Some(ref mut user_addr) => {
                                let _ = user_addr.do_send(Payload::new("Some one join the room"));
                            }
                            None => {
                                client.address.replace(msg.addr);
                            }
                        }
                        target.append_client(msg.client_id);
                        return MessageResult(JoinOutput::Success);
                    }
                } else {
                    println!("Cannot find Client in server!");
                    return MessageResult(JoinOutput::Rejected(RoomRejection::UnknownUser));
                }
            }
            None => {
                // return MessageResult(JoinOutput::Failed(FailureReason::COLLISION));
                return MessageResult(JoinOutput::Rejected(RoomRejection::UnknownRoom));
            }
        }
    }
}
