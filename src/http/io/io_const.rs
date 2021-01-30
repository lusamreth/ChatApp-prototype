// register error!
pub const BADFORMAT: &'static str = "/badformat";
pub const EMPTY: &'static str = "/empty";
pub const LENGTHLIMIT: &'static str = "/length_limit";
pub const REGISTRATION_ERR: &'static str = "/registration_error";

// internal message
pub const INTERNALFAILURE: &'static str = "/internal_error";
pub const ARERROR: &'static str = "/access_read";
pub const AWERROR: &'static str = "/access_write";
pub const COLLISION: &'static str = "/data_collision";

// room creation error
pub const ROOM_ERROR: &'static str = "/room_error";
pub const UNACCEPTABLE: &'static str = "/unacceptable";
pub const CREATION_MESSAGE: &'static str = "New user has been registered!";
pub const NA: &'static str = "N/A";

pub const SIGNAL_ERROR: &'static str = "/signal_error";

pub const ROOMERROR: &'static str = "/RoomError";
pub const USER_EXISTED: &'static str = "/ExistingUser";
pub const EXISTED_USER_MSG: &'static str = "User in the room is already Joined";
pub const UNKOWNUSER: &'static str = "/UnknownUser";
pub const UNKOWNROOM: &'static str = "/UnkownRoom";

pub const AUTH_ERROR: &'static str = "/authorization_err";
pub const EMPTY_HEADER: &'static str = "/EmptyHead";
pub const INVALIDTOKEN: &'static str = "/InvalidToken";
pub const EXPIRED_TOKEN: &'static str = "/ExpiredToken";
pub const EMPTY_COOKIE: &'static str = "/empty_cookie";
pub const PARSING_FAIL: &'static str = "/parsing_fail";
pub const BADJWT: &'static str = "/bad_jwt_components";
