import { Socket } from 'phoenix';


let socket = new Socket("/socket")

socket.connect()


let channel = socket.channel("room:1", {})

channel.join()
    .receive("ok", resp => { console.log("Joined successfully", resp) })
    .receive("error", resp => { console.log("Unable to join", resp) })

channel.push("test", { data: "join room:1" })
    .receive("ok", resp => { console.log("test receive:", resp) })
    .receive("error", resp => { console.log("test error:", resp) })

channel.push("test2", { data: "test message 2" })
    .receive("ok", resp => { console.log("test2 receive:", resp) })
    .receive("error", resp => { console.log("test2 error:", resp) })

channel.on("test3", resp => {
    console.log(resp);
})

// setTimeout(() => {
//     channel.leave()
// }, 1000)
