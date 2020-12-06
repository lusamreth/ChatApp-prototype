
function update_error(message){
    document.getElementById("error-panel").innerHTML = `<p class="error-txt">Error ${message} </p>`;
}

class Socket{
    constructor(url){
        this.url = url
    }
    
    init_socket(){
        this.detect_id();
        console.log(this.url);
        const ws = new WebSocket(this.url);
        return ws;
    }

    detect_id(){
        let user = JSON.parse(localStorage.getItem("user"));
        console.log(user);
        console.log("bruh");
        if (user) {
            this.url = `${this.url}/${user.user_id}`;
        }else {
            console.log("no data!");
            return;
        }
    }
}


const url = "ws://127.0.0.1:8030/ws";
const ws = new Socket(url).init_socket();
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


