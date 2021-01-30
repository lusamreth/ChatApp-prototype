use super::Server;
use crate::domain::*;
use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

impl Handler<Registration> for Server {
    type Result = MessageResult<Registration>;
    fn handle(&mut self, msg: Registration, _: &mut Self::Context) -> Self::Result {
        let result = self.handle_register(msg.username, msg.password);
        MessageResult(result)
    }
}

impl Handler<ListUser> for Server {
    type Result = MessageResult<ListUser>;
    fn handle(&mut self, _: ListUser, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.retrieve_users())
    }
}

impl Handler<CreateRoom> for Server {
    type Result = MessageResult<CreateRoom>;
    // type Result = RoomCreation;
    fn handle(&mut self, msg: CreateRoom, _: &mut Self::Context) -> Self::Result {
        let res = self.create_room(msg.name, msg.capacity, msg.creator.clone());
        println!("{:#?}", res);
        match res.handle {
            Some(id) => {
                self.send_message(id, "New Room has been created!", msg.creator);
            }
            None => {}
        }
        MessageResult(res)
    }
}

impl Handler<RoomRetrieval> for Server {
    type Result = Result<HashMap<Uuid, Room>, FailureReason>;

    fn handle(&mut self, _: RoomRetrieval, _: &mut Self::Context) -> Self::Result {
        self.retrieve_rooms()
    }
}

impl Handler<SignalInput> for Server {
    type Result = MessageResult<SignalInput>;

    fn handle(&mut self, mut msg: SignalInput, _: &mut Self::Context) -> Self::Result {
        match msg.code {
            SignalCode::Connect => MessageResult(self.connect(msg.id, &mut msg.addr, msg.room_id)),
            SignalCode::Disconnect => MessageResult(self.diconnect(msg.id)),
            SignalCode::Pending => todo!(),
        }
    }
}

impl SignalController for Server {
    fn connect(
        &mut self,
        user: Uuid,
        socket_addrs: &Recipient<Payload>,
        room_id: Uuid,
    ) -> SignalOutput {
        let missing_err = SignalOutput {
            status: ConnectionStatus::Aborted(AbortReason::UNACCEPTABLE(
                RoomRejection::UnknownUser,
            )),
            signaled_at: utility::timestamp_now(),
        };
        println!("room id : {}", room_id);
        println!("id : {}", user);
        if let Some(room) = self.rooms.read().unwrap().get(&room_id) {
            if let Some(_) = room.client_ids.iter().find(|id| user == **id) {
                match self.clients.write() {
                    Ok(mut handle) => match handle.get_mut(&user) {
                        Some(user) => {
                            user.address.replace(socket_addrs.clone());
                            SignalOutput {
                                status: ConnectionStatus::Connected,
                                signaled_at: utility::timestamp_now(),
                            }
                        }
                        None => {
                            println!("Non existent user!");
                            missing_err
                        }
                    },
                    Err(_) => {
                        println!("bad handle!");
                        SignalOutput {
                            status: ConnectionStatus::Aborted(AbortReason::Internal(
                                FailureReason::ACCESSWRITEERROR,
                            )),
                            signaled_at: utility::timestamp_now(),
                        }
                    }
                }
            } else {
                missing_err
            }
        } else {
            return SignalOutput {
                status: ConnectionStatus::Aborted(AbortReason::UNACCEPTABLE(
                    RoomRejection::UnknownRoom,
                )),
                signaled_at: utility::timestamp_now(),
            };
        }
    }

    fn diconnect(&mut self, user: Uuid) -> SignalOutput {
        match self.clients.write() {
            Ok(mut handle) => match handle.get_mut(&user) {
                Some(client) => {
                    client.address = None;
                    SignalOutput {
                        status: ConnectionStatus::Disconnected,
                        signaled_at: utility::timestamp_now(),
                    }
                }
                None => SignalOutput {
                    status: ConnectionStatus::Aborted(AbortReason::External(RefusedReason::EMPTY)),
                    signaled_at: utility::timestamp_now(),
                },
            },
            Err(_) => SignalOutput {
                status: ConnectionStatus::Aborted(AbortReason::Internal(
                    FailureReason::ACCESSREADERROR,
                )),
                signaled_at: utility::timestamp_now(),
            },
        }
    }

    fn pending(&mut self, _: Recipient<Payload>) -> SignalOutput {
        todo!()
    }
}
