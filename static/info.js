let Url = new URL(document.URL);

async function fetch_ws() {
    try {
        let response_ws = await fetch(Url, { headers: { 'If-Match': 'ws' } });
        let text_ws = await response_ws.text()
        return text_ws;
    }
    catch (e) {
        console.error("Error at fetching ws:", e)
    }

}

async function fetch_cloc() {
    try {
        let response_cloc = await fetch(Url,
            {
                headers: {
                    'If-Match': 'cloc'
                }
            });

        return response_cloc.text();
    } catch (e) {
        console.error("Error at fetching cloc", e)
    }
}

async function start(_e) {
    let cloc_promise = fetch_cloc();

    let websocket;
    try {
        let address = await fetch_ws();
        let url = "ws://" + document.location.host + document.location.pathname + address + document.location.pathname
        websocket = new WebSocket(url);
        startStreaming(websocket)
    }
    catch (e) {
        console.error("Error at ws:", e)
    }

    try {
        let cloc = await cloc_promise;
        if (cloc.length > 0) {
            stopStreaming(websocket);
            createTableFromResponse(cloc);

        }
    }
    catch (e) {
        console.error("Error at getting cloc promise", e)
    }
}

document.onload = start

async function stopStreaming(ws) {
    await ws.close()
}

function startStreaming(ws) {
    let send_ping = function () {
        if (ws.readyState !== WebSocket.CLOSING || ws.readyState !== WebSocket.CLOSED) {
            ws.send("ping")
        }
    }

    ws.onopen = function (event) {
        console.log("event", event);
        setInterval(send_ping, 500);
    }

    ws.onclose = function (event) {
        console.log("event", event);
        document.getElementById("hint").innerText = "Counting lines of code"
    }

    ws.onmessage = function (event) {
        let p = event.data

        // status.innerText = p;
        let lines = p.split(/\r?\n/)
        for (let i = 0; i < lines.length; ++i) {
            let payload = lines[i];
            // console.log("payload line:", payload)
            if (payload.includes("Cloning")) {
                document.getElementById("hint").innerHTML = "Cloning repository into server"
                document.getElementById("clone").innerText = "git clone https:/" + document.location.pathname
                document.getElementById("cloning").innerText = payload
            }
            if (payload.includes("Enumerating")) {
                let parts = payload.split(":");
                if (parts.length >= 3) {

                    // console.log(percent)
                    // document.getElementById("pg_enumerating").style.width = percent;
                    document.getElementById("enumerating").innerText = "remote: Enumerating objects:" + parts[parts.length - 1]
                }
            }
            if (payload.includes("Counting")) {
                let parts = payload.split(":");
                if (parts.length >= 3) {
                    let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g)[0])
                    // console.log("counting", percent)
                    percent = percent * 2 / 100
                    document.getElementById("pg_counting").style.width = percent + '%';
                    document.getElementById("counting").innerText = "remote: Counting objects:" + parts[parts.length - 1]

                }
            }
            if (payload.includes("Compressing")) {
                let parts = payload.split(":");
                if (parts.length >= 3) {
                    let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g))
                    // console.log("compressing", percent)
                    percent = percent * 15 / 100
                    document.getElementById("pg_compressing").style.width = percent + '%';
                    document.getElementById("compressing").innerText = "remote: Compressing objects:" + parts[parts.length - 1]
                }
            }
            if (payload.includes("Total")) {
                let parts = payload.split(":");
                if (parts.length >= 2) {
                    document.getElementById("total").innerText = "remote:" + parts[parts.length - 1]
                }
            }
            if (payload.includes("Receiving")) {
                let parts = payload.split(":");
                if (parts.length >= 2) {
                    let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g)[0]);
                    percent = percent * 70 / 100
                    // console.log("receving ", percent)
                    document.getElementById("pg_receiving").style.width = percent + '%';
                    document.getElementById("receiving").innerText = "Receiving objects:" + parts[parts.length - 1];
                }
            }
            if (payload.includes("Resolving")) {
                let parts = payload.split(":");
                if (parts.length >= 2) {
                    let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g))
                    console.log("resolving", percent)
                    percent = percent * 2 / 100
                    document.getElementById("pg_resolving").style.width = percent + '%';
                    document.getElementById("resolving").innerText = "Resolving deltas:" + parts[parts.length - 1]
                }
            }
            if (payload.includes("Updating")) {
                if (payload.includes("done")) {
                    document.getElementById("hint").innerText = "Counting lines of code"
                }
                let parts = payload.split(":");
                if (parts.length >= 2) {
                    let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g))
                    console.log("updating", percent)
                    percent = percent * 11 / 100
                    document.getElementById("pg_updating").style.width = percent + '%';
                    document.getElementById("updating").innerText = "Updating objects:" + parts[parts.length - 1]
                }
            }
        }
    }
}

function createTableFromResponse(data) {
    console.log(data)
    let strings = data.split("\n")
    document.getElementById("processing").hidden = true
    strings.splice(0, 1);
    strings.splice(1, 1);
    strings.splice(-3)

    let processed = strings.splice(-1, 1)
    console.log(processed)
    strings.splice(-1)
    let cocomo = strings.splice(-3, 3);
    strings.splice(-1)
    strings.splice(-2, 1)

    for (let i = 0; i < strings.length; ++i) {
        let array = strings[i].trim().split(/\s+/);         
        while (array.length > 7) {
            array[0] += array[1]
            array.splice(1, 1)
        }
        strings[i] = array;
    }

    let table = '<table class="table">'
    table += createTableThead(strings[0])
    table += "<tbody>"

    for (let i = 1; i < strings.length; ++i) {
        table += createTableRow(strings[i])
    }


    table += "</tbody>"
    table += "</table>"
    document.getElementById("t").innerHTML = table
    console.log(strings, cocomo)
    createCocomoFromResponse(cocomo, processed)
}

function createTableThead(array) {
    let thead = '<thead style="border-color:black;"><tr>'
    for (let i = 0; i < array.length; ++i) {
        thead += '<th scope="col">' + array[i] + '</th>'
    }
    thead += "</tr></thead>"
    return thead;
}


function createTableRow(array) {
    let row = "<tr>"

    row += '<th scope="row">' + array[0] + '</th>'

    for (let i = 1; i < array.length; ++i) {
        row += "<td>" + array[i] + "</td>"
    }
    row += "</tr>"
    return row
}

function createCocomoFromResponse(cocomo, processed) {
    let str = ""
    str += '<h6>' + processed + '</h6>'
    str += '<hr class="bg-primary"></hr>'
    str += '<h6>' + cocomo[0] + '</h6>' + '<h6>' + cocomo[1] + '</h6>' + '<h6>' + cocomo[2] + '</h6>';
    console.log(str)
    document.getElementById("cocomo").innerHTML = str
}

document.addEventListener("DOMContentLoaded", start);