let Url = new URL(document.URL);

async function fetch_ws() {
    let response_ws = await fetch(Url, { headers: { 'If-Match': 'ws' } });
    return extractContent(response_ws, "Error at fetching websocket")
}

async function fetch_cloc() {
    let response_cloc = await fetch(Url, { headers: { 'If-Match': 'cloc' } });
    return extractContent(response_cloc, "Error at fetching lines of code");
}

function extractContent(response, error_msg) {
    message = error_msg ? error_msg + ":\n" : ""
    const contentType = response.headers.get("content-type");
    if (contentType && contentType.indexOf("application/json") !== -1) {
        return response.json().then(data => {
            return data
        });
    } else {
        if (response.status >= 400) {
            return response.text().then(text => {
                throw new FetchError(response.status, message + text)
            });
        }
        else {
            return response.text().then(text => {
                return text
            });
        }
    }
}

async function fetch_branch_info(url_str) {
    let branch_api_info_url = new URL(url_str)
    console.log("branch_api_info_url", branch_api_info_url)
    let response_branch_info = await fetch(branch_api_info_url);
    return extractContent(response_branch_info, "Error at fetching default branch")
}

async function fetch_branch_commit(url_str) {
    let branch_commit = new URL(url_str)
    console.log("branch_commit", branch_commit)
    let response = await fetch(branch_commit);
    return extractContent(response, "Error at fetching branch commit")
}

async function preparePage(url) {
    let path_name_array = url.pathname.split('/').filter(item => item);
    console.log(path_name_array, url)
    if (path_name_array.length < 3) {
        console.error("Incorrect URL:", url)
        throw new PrepareError("Incorrect URL", url + " should contain repository hostname, owner and repository name");
    }
    let repository_hostname = path_name_array[0];
    let onwer = path_name_array[1]
    let repository_name = path_name_array[2]
    let branch = ""


    try {
        let origin_url = "https:/" + Url.pathname
        console.log("origin_url", origin_url)
        let parsed_url = gitUrlParse(origin_url)
        console.log("parsed_url", parsed_url);
        let img = ''
        if (repository_hostname == "github.com") {
            img = '<img alt="Open repository" src="/static/GitHub-Mark-32px.png" class="float-start">'
            if (repository_name.slice(-4) === ".git") { repository_name = repository_name.slice(0, -4) }
        }
        else if (repository_hostname == "gitlab.com") {
            img = '<img alt="Open repository" src="/static/gitlab32.png" class="float-start">'
            if (repository_name.slice(-4) === ".git") { repository_name = repository_name.slice(0, -4) }
        }
        else if (repository_hostname == "bitbucket.org") {
            img = '<img alt="Open repository" src="/static/bitbucket.png" class="float-start">'
            if (repository_name.slice(-4) === ".git") { repository_name = repository_name.slice(0, -4) }
        }
        else {
            img = '<img alt="Open repository" src="/static/git32.png" class="float-start">'
            if (repository_name.slice(-4) !== ".git") { origin_url += ".git" }
        }
        let pic_ref = '<a target="_blank" rel="noopener noreferrer canonical" href="' + origin_url + '">' + img + '</a>'
        let show_url = "https://" + repository_hostname + '/' + onwer + '/' + repository_name
        console.log("repository_name", repository_name)
        for (let i = 3; i < path_name_array.length; ++i) { show_url += '/' + path_name_array[i] }
        document.getElementById("url").innerText = show_url
        document.getElementById("url").setAttribute("href", origin_url)
        document.getElementById("url_pic").innerHTML = pic_ref
    }
    catch (e) {
        throw new Error("Can't setup URL" + e)
    }

    if (path_name_array[3] === undefined) {
        branch = await fetch_branch_info(Url.protocol + Url.host + "/api" + Url.pathname)
        document.getElementById("branch").innerText = branch.default_branch
        let commit = await fetch_branch_commit(Url.protocol + Url.host + "/api" + Url.pathname + "/tree/" + branch.default_branch)
        console.log("commit", commit)
        document.getElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname === "github.com" && path_name_array[3] === "tree" && path_name_array[4] !== undefined) {
        for (let i = 4; i < path_name_array.length; ++i) {
            console.log("el:", path_name_array[i])
            branch += '/' + path_name_array[i]
        }
        document.getElementById("branch").innerText = branch.slice(1)
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + onwer + "/" + repository_name + "/tree" + branch
        let commit = await fetch_branch_commit(url_str)
        document.getElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname === "gitlab.com") {
        if (path_name_array[3] === "tree" && path_name_array[4] !== undefined) {
            for (let i = 4; i < path_name_array.length; ++i) {
                console.log("el:", path_name_array[i])
                branch += '/' + path_name_array[i]
            }
        }
        else if (path_name_array[3] === "-" && path_name_array[4] === "tree" && path_name_array[5] !== undefined) {
            for (let i = 5; i < path_name_array.length; ++i) {
                console.log("el:", path_name_array[i])
                branch += '/' + path_name_array[i]
            }
        }
        else {
            let error_msg = url + "\nAfter tree/ must be followed by a branch name"
            console.error(error_msg)
            throw new PrepareError("Incorrect URL", error_msg)
        }
        document.getElementById("branch").innerText = branch.slice(1)
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + onwer + "/" + repository_name + "/tree" + branch
        let commit = await fetch_branch_commit(url_str)
        document.getElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname === "bitbucket.org" && path_name_array[3] === "src" && path_name_array[4] !== undefined) {
        for (let i = 4; i < path_name_array.length; ++i) {
            console.log("el:", path_name_array[i])
            branch += '/' + path_name_array[i]
        }
        document.getElementById("branch").innerText = branch.slice(1)
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + onwer + "/" + repository_name + "/src" + branch
        let commit = await fetch_branch_commit(url_str)
        document.getElementById("commit").innerText = commit.commit
    }
    else {
        let error_msg = url + "\nAfter tree/ must be followed by a branch name"
        console.error(error_msg)
        throw new PrepareError("Incorrect URL", error_msg)
    }

    return true
}

function showError(status, message) {
    console.error(message)
    console.trace()
    document.getElementById("alert_block").classList.toggle('show')
    document.getElementById("alert_message").innerText = message ? message : ""
    document.getElementById("error_status").innerText = status ? status : ""
    document.getElementById("repository").hidden = true
    document.getElementById("processing").hidden = true
}

async function start(_e) {
    let ok = false
    try {
        ok = await preparePage(new URL(document.URL))

        if (!ok) { return; }
        let ws_json = await fetch_ws();
        let cloc_promise = fetch_cloc();
        console.log(ws_json)
        let url = document.location.host + "/ws" + document.location.pathname
        let websocket;
        console.log("protocol", document.location.protocol)
        if (document.location.protocol === "https:") {
            websocket = new WebSocket("wss://" + url)
        }
        else {
            websocket = new WebSocket("ws://" + url)
        }

        console.log("websocket:", url);
        console.log(websocket)
        // websocket.addEventListener('error', function (event) {
        //     console.log('WebSocket error: ', event);
        //     stopStreaming(websocket);
        // });
        startStreaming(websocket)
        let cloc = await cloc_promise;
        // console.log("cloc", cloc)
        if (cloc.length > 0) {
            stopStreaming(websocket)
            createTableFromResponse(cloc);
        }
    }
    catch (err) {
        if (err instanceof FetchError || err instanceof PrepareError) {
            showError(err.status, err.message)
        } else {
            showError(err)
        }
    }
}

document.onload = start

async function stopStreaming(ws) {
    return await ws.close()
}

function startStreaming(ws) {
    let send_ping = function () {
        if (ws.readyState === WebSocket.OPEN) {
            // console.log("ping ws")
            ws.send("ping")
        }
    }

    ws.onopen = function (event) {
        console.log("open ws", event);
        setInterval(send_ping, 500);
    }

    ws.onclose = function (event) {
        console.log("event", event);
        document.getElementById("hint").innerText = "Counting lines of code"
        console.log("WEB SOCKET  CLOSED");
    }

    ws.onmessage = function (event) {
        let p = event.data

        // status.innerText = p;
        let lines = p.split(/\r?\n/)
        for (let i = 0; i < lines.length; ++i) {
            let payload = lines[i];
            console.log("payload line:", payload)
            if (payload.includes("Done")) {
                fetch_cloc().then((cloc) => {
                    if (cloc.length > 0) {
                        stopStreaming(ws)
                        createTableFromResponse(cloc);
                    }
                })
            }
            // document.getElementById("status").innerText += payload
            document.getElementById("hint").innerHTML = "Downloading repository into server"
            if (payload.includes("Cloning")) {
                document.getElementById("clone").innerText = "git clone https:/" + document.location.pathname
                document.getElementById("cloning").innerText = payload
            }
            // if (payload.includes(""))
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
                    percent = percent * 1 / 100
                    document.getElementById("pg_counting").style.width = percent + '%';
                    document.getElementById("counting").innerText = "remote: Counting objects:" + parts[parts.length - 1]

                }
            }
            if (payload.includes("Compressing")) {
                let parts = payload.split(":");
                if (parts.length >= 3) {
                    let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g))
                    // console.log("compressing", percent)
                    percent = percent * 16 / 100
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

function createCocomoFromResponse(cocomo_data) {
    let str = ""

    str += '<div class="card-body"><h5 class="card-title"><strong>COCOMO</strong></h5><h6 class="card-subtitle mb-4 text-muted">Constructive Cost Model (<a target="_blank" rel="noopener noreferrer canonical" href="https://en.wikipedia.org/wiki/COCOMO">wiki</a>)</h6>'
    str += '<p class="card-text">' + cocomo_data[0] + '</p>'
    str += '<p class="card-text">' + cocomo_data[1] + '</p>'
    str += '<p class="card-text">' + cocomo_data[2] + '</p>'
    str += '</div>'

    let cocomo = document.getElementById("cocomo");
    cocomo.innerHTML = str
    cocomo.hidden = false

}

// function createAboutInfo(url_s, branch_s, commit_s) {
//     let url_str = url_s.split(' ')[1]
//     let ref = '<a target="_blank" rel="noopener noreferrer canonical" href="' + url_str + '">' + url_str + '</a>'

//     let urlRow = '<div class="row align-items-center">'
//     urlRow += '<div class="col col-sm-auto"><strong>URL:</strong></div>'
//     urlRow += '<div class="col col-sm-auto text-truncate float-start">' + ref + '</div>'

//     if (url_str.includes("github.com")) {
//         let img = '<img alt="Open repository" src="/static/GitHub-Mark-32px.png" class="float-start">'
//         let ref = '<a target="_blank" rel="noopener noreferrer canonical" href="' + url_str + '">' + img + '</a>'
//         urlRow += '<div class="col col-sm-auto">' + ref + '</div>'
//     }

//     urlRow += '</div>'
//     console.log(urlRow)

//     let branch_str = branch_s.split(' ')[1]
//     let branchRow = '<div class="row pt-2">'
//     branchRow += '<div class="col col-sm-auto"><strong>Branch:</strong></div>'
//     branchRow += '<div class="col">' + branch_str + '</div>'
//     branchRow += '</div> '

//     let commit_str = commit_s.split(' ')[1]
//     let commitRow = '<div class="row pt-2"><div class="col col-sm-auto">Commit:</div><div class="col text-truncate float-start">' + commit_str + '</div></div>'

//     let res = urlRow + branchRow + commitRow;
//     document.getElementById("about").innerHTML = res
// }

document.addEventListener("DOMContentLoaded", start);

class FetchError extends Error {
    constructor(status, message) {
        super(message);
        this.name = "FetchError";
        this.status = status
    }
}

class PrepareError extends Error {
    constructor(status, message) {
        super(message);
        this.name = "PrepareError";
        this.status = status
    }
}
