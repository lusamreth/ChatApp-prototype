use super::*;

// display for debugging
impl<T: Serialize + std::fmt::Debug + Clone> std::fmt::Display for ErrResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "et:{:#?}\nst:{:#?}\ndts:{:#?}\ninst:{:#?}",
            self.error_type, self.sub_type, self.details, self.instance
        )
    }
}
pub fn text_error(refusal: &RefusedReason) -> &'static str {
    match refusal {
        RefusedReason::BADFORMAT => BADFORMAT,
        RefusedReason::EMPTY => EMPTY,
    }
}

impl From<&FailureReason> for ResponseFeildError {
    fn from(reason: &FailureReason) -> Self {
        ResponseFeildError {
            field: NA.to_string(),
            error: reason.to_string(),
        }
    }
}

impl From<&RefusedReason> for ResponseFeildError {
    fn from(reason: &RefusedReason) -> Self {
        ResponseFeildError {
            field: "N/A".to_string(),
            error: reason.to_string(),
        }
    }
}

impl From<&RoomRejection> for ResponseFeildError {
    fn from(reason: &RoomRejection) -> Self {
        ResponseFeildError {
            field: "N/A".to_string(),
            error: reason.to_string(),
        }
    }
}

pub fn internal_failure(reason: &FailureReason) -> &'static str {
    match reason {
        crate::domain::FailureReason::ACCESSWRITEERROR => AWERROR,
        crate::domain::FailureReason::COLLISION => COLLISION,
        crate::domain::FailureReason::ACCESSREADERROR => ARERROR,
    }
}
