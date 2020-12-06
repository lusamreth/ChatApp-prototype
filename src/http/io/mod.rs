mod io_const;
use crate::domain::*;
pub use io_const::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
mod converter;
use converter::*;
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInput {
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationOutput {
    pub user_id: uuid::Uuid,
    pub creation_date: Duration,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagePayload {
    pub message: String,
    pub date: Duration,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrRepsonse<T> {
    pub error_type: URI,
    pub sub_type: PARTIALURI,
    #[serde(flatten)]
    pub details: T,
    pub instance: String,
}
pub trait Responsive {
    fn map_err(&self) -> String;
    fn to_json(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseFeildError {
    pub field: String,
    pub error: String,
}

impl Responsive for RegisterRes {
    fn map_err(&self) -> String {
        let mut stat = String::new();
        let resp_err: ResponseFeildError;
        // subtype
        let payload_st = match &self.status {
            RegistrationStatus::REFUSED(refusal, field) => {
                stat.push_str(REGISTRATION_ERR);
                let r = refusal.to_string();
                let field = String::from(&*field.clone());
                let field_error = format!(r#"field - {} : {}"#, &field, r);
                resp_err = ResponseFeildError {
                    field,
                    error: field_error,
                };

                text_error(refusal)
            }
            RegistrationStatus::FAILED(reason) => {
                resp_err = ResponseFeildError::from(reason);
                internal_failure(reason)
            }
            RegistrationStatus::CREATED => panic!("WRONG SEMATIC! \n Creation is not error!"),
        };
        let result_err = ErrRepsonse {
            error_type: stat,
            sub_type: payload_st.to_string(),
            details: resp_err,
            instance: NA.to_string(),
        };
        return serde_json::to_string(&result_err).expect("JSON SEMATIC FAILURE!");
    }

    fn to_json(&self) -> String {
        match self.status {
            RegistrationStatus::CREATED => {}
            _ => panic!("Unexpected status code! ABORT"),
        };
        if let None = self.cl_id {
            panic!("expect client id! probably an error payload!");
        }
        let res = RegistrationOutput {
            user_id: self.cl_id.unwrap(),
            creation_date: crate::domain::utility::timestamp_now(),
            message: CREATION_MESSAGE.to_string(),
        };
        return serde_json::to_string(&res).expect("bad parsing payload");
    }
}

type URI = String;
type PARTIALURI = String;

#[derive(Debug, Deserialize, Serialize)]
struct RoomInput {
    room_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomCreationOutput {
    pub room_id: uuid::Uuid,
    pub created_at: Duration,
    pub message: String,
}

impl Responsive for RoomCreation {
    fn map_err(&self) -> String {
        let mut resp;
        let status = match &self.status {
            RoomCreationStatus::CREATED => panic!("expect error payload"),
            RoomCreationStatus::ERROR(room_err) => {
                match room_err {
                    RoomError::UNACCEPTABLE(report) => {
                        resp = ResponseFeildError {
                            field: String::new(),
                            error: report.to_string(),
                        };
                        UNACCEPTABLE
                    }
                    RoomError::REFUSED(reason) => {
                        resp = ResponseFeildError::from(reason);
                        handle_roomrejection(reason)
                    }
                    RoomError::INTERNALERROR(failure) => {
                        // resp = ResponseFeildError::from(failure);
                        let stat = internal_failure(failure);
                        return stat.to_string();
                    }
                }
            }
        };
        resp.field = "Room_name".to_string();
        let rc = ErrRepsonse {
            error_type: ROOM_ERROR.to_string(),
            instance: NA.to_string(),
            sub_type: status.to_string(),
            details: resp,
        };
        serde_json::to_string(&rc).expect("bad parsing json!")
    }

    fn to_json(&self) -> String {
        let id;
        if let Some(rid) = self.handle {
            id = rid;
        } else {
            panic!("The item is an error payload!");
        };
        let output = RoomCreationOutput {
            room_id: id,
            created_at: utility::timestamp_now(),
            message: "new room has been created".to_string(),
        };
        serde_json::to_string(&output).unwrap()
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Rooms {
    pub id: uuid::Uuid,
    pub room_name: String,
    pub participants: usize,
    pub capacity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalPayload {
    pub signaled_at: Duration,
    pub status: String,
    pub message: String,
}
impl Responsive for SignalOutput {
    fn map_err(&self) -> String {
        let mut details;
        let state_type = match &self.status {
            ConnectionStatus::Aborted(error) => match error {
                AbortReason::Internal(fr) => {
                    details = ResponseFeildError::from(fr);
                    internal_failure(fr)
                }
                AbortReason::External(ext) => {
                    if let RefusedReason::EMPTY = ext {
                        details = ResponseFeildError {
                            field: "RoomId".to_string(),
                            error: "Cannot find the target room!".to_string(),
                        };
                        UNKOWNROOM
                    } else {
                        details = ResponseFeildError::from(ext);
                        details.field = "signal_input".to_string();
                        text_error(ext)
                    }
                }
                AbortReason::UNACCEPTABLE(room_rjt) => {
                    println!("P{:#?}", room_rjt);
                    details = ResponseFeildError {
                        field: "RoomId".to_string(),
                        error: room_rjt.to_string(),
                    };

                    handle_roomrejection(room_rjt)
                }
            },
            _ => panic!("This is a success payload!"),
        };

        let back = ErrRepsonse {
            error_type: SIGNAL_ERROR.to_string(),
            sub_type: state_type.to_string(),
            details,
            instance: NA.to_string(),
        };
        serde_json::to_string(&back).expect("signal payload parsing error!")
    }

    fn to_json(&self) -> String {
        // extract status
        let status = match self.status {
            ConnectionStatus::Connected => String::from("Connected"),
            ConnectionStatus::Disconnected => String::from("Disconnected"),
            _ => panic!("This is not a success payload!"),
        };
        let payload = SignalPayload {
            status,
            signaled_at: self.signaled_at,
            message: self.status.to_string(),
        };
        serde_json::to_string(&payload).expect("signal payload parsing error!")
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct JoinResp {
    pub message: String,
    pub status: String,
}
pub fn handle_roomrejection(reason: &RoomRejection) -> &str {
    // reason is the roomrejection domain
    match reason {
        RoomRejection::UnknownRoom => UNKOWNROOM,
        RoomRejection::UnknownUser => UNKOWNUSER,
        RoomRejection::Reject(reason) => text_error(reason),
    }
}
impl Responsive for JoinOutput {
    fn map_err(&self) -> String {
        let failing_case;
        let err_tp = match self {
            JoinOutput::Rejected(reason) => {
                failing_case = ResponseFeildError::from(reason);
                handle_roomrejection(reason)
            }
            JoinOutput::Failed(failing) => {
                failing_case = ResponseFeildError::from(failing);
                if let FailureReason::COLLISION = failing {
                    USER_EXISTED
                } else {
                    internal_failure(failing)
                }
            }
            _ => panic!("Not an error payload"),
        };

        let resp = ErrRepsonse {
            error_type: ROOMERROR.to_string(),
            sub_type: err_tp.to_string(),
            details: failing_case,
            instance: NA.to_string(),
        };

        serde_json::to_string(&resp).expect("bad parsing!")
    }

    fn to_json(&self) -> String {
        match self {
            JoinOutput::Success => {
                let r = JoinResp {
                    message: "User sucessfully joined the room!".to_string(),
                    status: "Success".to_string(),
                };
                serde_json::to_string(&r).expect("bad error parsing!")
            }
            _ => panic!("Error payload!"),
        }
    }
}
