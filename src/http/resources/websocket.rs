use crate::backend::Server;
use crate::pipe;
use actix::Addr;
use actix_web::{http, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;
// main lobby!
const MAIN: &'static str = "eb44e5e4-b3c2-4021-b8a4-5081e5c1af81";

pub async fn chat_route(
    req: HttpRequest,
    //param: web::Path<String>,
    stream: web::Payload,
    srv: web::Data<Addr<Server>>,
) -> Result<HttpResponse, Error> {
    let mut uid = None;
    let mut error = None;
    match req.match_info().get("socket_id") {
        Some(sc_id) => match Uuid::parse_str(&sc_id) {
            Ok(id) => uid = Some(id),
            Err(_) => {
                error = Some("Fail to parse the given id!");
            }
        },
        None => {
            uid = Some(Uuid::new_v4());
        }
    }

    match error {
        Some(error) => {
            let mut err_resp = HttpResponse::build(http::StatusCode::BAD_REQUEST);
            let z = err_resp.body(error);
            let axti: actix_web::Error = z.into();
            Err(axti)
        }
        None => {
            let newpipe = pipe::SimpleSocket::new(
                srv.get_ref().clone().into(),
                Uuid::parse_str(MAIN).expect("bad id"),
                uid.unwrap(),
            );
<<<<<<< HEAD
            ws::start(newpipe, &req, stream)
=======
<<<<<<< HEAD
            ws::start(newpipe, &req, stream)
=======
            let a = req.headers().get("sec-websocket-protocol");
            
            let prot = ["token"];
            ws::start_with_protocols(newpipe,&prot,&req, stream)
>>>>>>> 21fb43b (Handshake authentication)
>>>>>>> d41459f (Improving authentication logic!)
        }
    }
}
