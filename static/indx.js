let ws = new WebSocket("ws://127.0.0.1:8030/ws");
ws.onopen = (e) => {
    ws.send("users");
}
// setInterval(() => {
//     ws.send("d");
// }, 1000);
console.log('working');
ws.onmessage = (e) => {
    console.log('this');
    console.log(e);
}

const fetchUsers = () => {
    axios.get('https://reqres.in/api/users')
        .then(response => {
            const users = response.data.data;
            console.log(`GET list users`, users);
        })
        .catch(error => console.error(error));
};

fetchUsers();