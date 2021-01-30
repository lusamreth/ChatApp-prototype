
function update_error(message){
    document.getElementById("error-panel").innerHTML = `<p class="error-txt">Error ${message} </p>`;
}

function base64_url(base64){

    let re = new RegExp('/+/',"g");
    let re2 = new RegExp('/[\/]/g',"g");
    return base64.replace(re,"-").replace(re2,"_");
}

class Socket{
    constructor(url){
        this.url = url
        this.token = ""
    }
    
    init_socket(){
        this.detect_id();
        console.log(this.url);

        let h1,h2;
        let protocol = [];
        
        // not json!
        // server will produce 44 chars csrf-token[opaque-random]
        let csrf = localStorage.getItem("csrf");
        console.log(csrf);        
        if (csrf !== null){
            csrf = base64_url(csrf);
            let l = csrf.length;
            h1 = csrf.slice(0,l/2);
            h2 = csrf.slice(l/2,l-1);

            protocol =["token",h1,h2];
        }
        // base 64 with padding will not work!!
        console.log(h1);
        console.log(h2);
        return new WebSocket(this.url,protocol);
    }

    detect_id(){
        let user = JSON.parse(localStorage.getItem("user"));
        if (user) {
            this.url = `${this.url}/${user.user_id}`;
        }else {
            console.log("no data!");
            return;
        }
    }
}

const url = "ws://localhost:8030/ws";
const ws = new Socket(url).init_socket();
ws.onopen = (s) => {
    ws.send("room:create_room/apsoc");
    ws.onmessage = (e) => {
        console.dir(e.data);
    }
}
//const ws = new WebSocket(url);
function detect_err(payload){
    if (typeof payload === "object") {
        return Object.keys(payload).find(key => key === "error")
    }else if (typeof payload === "string" ){
        return true;
    }
}

function built_ws_btn(button_name,built_closure){
    let submit_btn = document.getElementById(button_name);
    return function ws_btn(req,param) {
        if (typeof req !== "string") {
            throw "Only support string message!";
        }
        submit_btn.addEventListener("click",(e) => {
            // retrieve from closure
            e.preventDefault();
            let input = document.getElementById(param).value;
            if (built_closure !== undefined){
                built_closure(req,input);
            }else {
                throw "Button require closure functionality";
            }
        })
    }
}

//count start from one!;
//let queue = [""];
// require to use in a try catch scope
function cont_req(queue,results){
    if (queue.length === 0) return results;
    ws.send(queue.pop());
    ws.onmessage = (payload) => {
        let payload_json = JSON.parse(payload.data);
        if (detect_err(payload_json)){
            console.log("there is error!");
            throw payload_json;
        }
        results.push(payload_json);
        console.log(results);
        cont_req(queue,results);
    }
}
// testing the continue request 
// so far passed!
function testcont(){
    console.log("testing cont");
    let test_queue = [
        "room:create_room/apoa",
        "room:create_room/dbeu",
        "room:create_room/comal"];
    let results = [];
    
    cont_req(test_queue,results);
    console.log(results);
}
function dea(){
    throw "apap";
}
function nua(){
    try{
        dea();
    }catch(e){
        console.log(e);
    }
    
}
function recur_push(item,array){
   array.push(item) ;
   recur_push(item,array);
}

export default {update_error}
