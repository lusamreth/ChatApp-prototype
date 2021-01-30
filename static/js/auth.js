//import axios from "../../node_modules/axios/dist/axios.min"; 
import axios from "axios";
import utils from "./utils";
function update_userstate(value){
    // reset the log in state
    document.getElementById("input").innerHTML = "";
    localStorage.setItem("user",JSON.stringify(value));
    JSON.parse(localStorage.getItem("user"));
}

let username = "username";
let submit_name = document.getElementById("submitname");
// error store in stack 
async function run_auth(pf,u,p){
    return await axios.post(
      `http://localhost:8030/api/v1/${pf}`,
      {
        username:u, 
        password:p,
      },
      {
        withCredentials:true,
        headers: {
          "Content-type": "application/json; charset=UTF-8",
          "Access-Control-Allow-Origin" : "*"
        }
      }
    ).then(e => {
        console.dir(e);
        return e
    })
}

const http_url = "http://127.0.0.1:8030/api/v1/login";

async function login(input){
    try {
        let payload = await run_auth("register",input.username,input.password).catch(e => {
            console.dir(e);
        });

        let csrf = payload.headers.csrf;
        console.log("this csrf" + csrf);
        let {user_id,creation_date} = payload.data;

        localStorage.setItem("csrf",csrf);
        update_userstate({
            name:input,// username
            user_id,
            creation_date
        });
        console.log(document.cookie);
        //window.location = "/static/rooms.html";

    }catch(e){
       console.log("error!");
       console.error(e);
    }
}
submit_name.addEventListener("click",(e) => {
    e.preventDefault();
    let username = document.getElementById("username").value;
    login({
        username,
        password:"Soasmdksj@08123"

    });
})
