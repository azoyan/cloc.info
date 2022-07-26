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
        let url = "ws://" + document.location.host + address + document.location.pathname 
        console.log("websocket:", url);
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
    let strings = data.split("\n")
    console.log(data)
    document.getElementById("processing").hidden = true
    strings.splice(0, 1);
    strings.splice(1, 1);
    console.log(strings.splice(-1, 1))
    let commit_array = strings.splice(-1, 1);
    let branch_array = strings.splice(-1, 1)
    let url_array = strings.splice(-1, 1);

    createAboutInfo(url_array[0], branch_array[0], commit_array[0])

    console.log(strings.splice(-2, 2))

    let processed = strings.splice(-1, 1)
    console.log(processed)
    console.log(strings.splice(-1, 1))
    let cocomo = strings.splice(-3, 3);
    console.log(cocomo)
    console.log(strings.splice(-1))
    console.log(strings.splice(-2, 1))

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
    let caption = '<caption>' + processed + '</caption>'
    table += caption
    table += "</table>"
    document.getElementById("t").innerHTML = table
    document.getElementById("t").hidden = false
    console.log(strings, cocomo)
    createCocomoFromResponse(cocomo)
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

function createCocomoFromResponse(cocomo) {
    let str = ""

   str += '<div class="card-body"><h5 class="card-title"><strong>COCOMO</strong></h5><h6 class="card-subtitle mb-4 text-muted">Constructive Cost Model (<a target="_blank" rel="noopener noreferrer canonical" href="https://en.wikipedia.org/wiki/COCOMO">wiki</a>)</h6>'
    str += '<p class="card-text">' + cocomo[0] + '</p>'
    str += '<p class="card-text">' + cocomo[1] + '</p>'
    str += '<p class="card-text">' + cocomo[2] + '</p>'
    str += '</div>'
    document.getElementById("cocomo").innerHTML = str
}

function createAboutInfo(url_s, branch_s, commit_s) {
    let url_str = url_s.split(' ')[1]
    let ref = '<a target="_blank" rel="noopener noreferrer canonical" href="' + url_str + '">' + url_str + '</a>'

    let urlRow = '<div class="row align-items-center">'
    urlRow += '<div class="col col-sm-auto"><strong>URL:</strong></div>'
    urlRow += '<div class="col col-sm-auto text-truncate float-start">' + ref + '</div>'

    if (url_str.includes("github.com")) {
        let img = '<img alt="Open repository" src="/static/GitHub-Mark-32px.png" class="float-start">'
        let ref = '<a target="_blank" rel="noopener noreferrer canonical" href="' + url_str + '">' + img + '</a>'
        urlRow += '<div class="col col-sm-auto">' + ref + '</div>'
    }

    urlRow += '</div>'
    console.log(urlRow)

    let branch_str = branch_s.split(' ')[1]
    let branchRow = '<div class="row pt-2">'
    branchRow += '<div class="col col-sm-auto"><strong>Branch:</strong></div>'
    branchRow += '<div class="col">' + branch_str + '</div>'
    branchRow += '</div> '

    let commit_str = commit_s.split(' ')[1]
    let commitRow = '<div class="row pt-2"><div class="col col-sm-auto">Commit:</div><div class="col text-truncate float-start">' + commit_str + '</div></div>'

    let res = urlRow + branchRow + commitRow;
    document.getElementById("about").innerHTML = res
}

document.addEventListener("DOMContentLoaded", start);