let Url = new URL(document.URL);
const LOGO = document.getElementById("logo")
let angle = 0
let interval;
function rotate() {
    angle += 4;
    angle %= 360;
    LOGO.setAttribute("transform", "rotate(" + angle + ")");
}
function stopRotate() {
    clearInterval(interval)
    LOGO.removeAttribute("transform")
}
async function fetch_ws() {
    let response_ws = await fetch(Url, { headers: { 'If-Match': 'ws' } });
    return extractContent(response_ws, "Error at fetching websocket")
}

async function fetch_cloc() {
    let response = await fetch(Url, { headers: { 'If-Match': 'cloc' } });
    return extractContent(response, "Error at fetching lines of code");
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
        else if (response.status === 202) {
            return 202;
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
    let owner = path_name_array[1]
    let repository_name = path_name_array[2]
    let branch = ""

    // try {
    let origin_url = "https:/" + Url.pathname
    console.log("origin_url", origin_url)
    let parsed_url = gitUrlParse(origin_url)
    console.log("parsed_url", parsed_url);
    let img = createRepositoryIcon(repository_hostname, 32, 32)

    // if (repository_name.slice(-4) === ".git") { repository_name = repository_name.slice(0, -4) }
    // else if (repository_name.slice(-4) !== ".git") { origin_url += ".git" }

    let pic_ref = '<a target="_blank" rel="noopener noreferrer canonical" href="' + origin_url + '">' + img + '</a>'
    let show_url = "https://" + repository_hostname + '/' + owner + '/' + repository_name
    console.log("repository_name", repository_name)
    for (let i = 3; i < path_name_array.length; ++i) { show_url += '/' + path_name_array[i] }
    document.getElementById("url").innerText = show_url
    document.getElementById("url").setAttribute("href", origin_url)
    document.getElementById("url_pic").innerHTML = pic_ref
    // }
    // catch (e) {
    //     throw new Error("Can't setup URL" + e)
    // }

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
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + owner + "/" + repository_name + "/tree" + branch
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
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + owner + "/" + repository_name + "/tree" + branch
        let commit = await fetch_branch_commit(url_str)
        document.getElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname === "bitbucket.org" && path_name_array[3] === "src" && path_name_array[4] !== undefined) {
        for (let i = 4; i < path_name_array.length; ++i) {
            console.log("el:", path_name_array[i])
            branch += '/' + path_name_array[i]
        }
        document.getElementById("branch").innerText = branch.slice(1)
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + owner + "/" + repository_name + "/src" + branch
        let commit = await fetch_branch_commit(url_str)
        document.getElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname == "codeberg.org" && path_name_array[3] === "src") {
        let index;
        if (path_name_array[4] === "branch" && path_name_array[5] !== undefined) {
            index = 5
        }
        else if (path_name_array[4] !== undefined) {
            index = 4
        }
        for (let i = index; i < path_name_array.length; ++i) {
            console.log("el:", path_name_array[i])
            branch += '/' + path_name_array[i]
        }

        document.getElementById("branch").innerText = branch.slice(1)
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + owner + "/" + repository_name + "/src" + branch
        let commit = await fetch_branch_commit(url_str)
        document.getElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname == "gitea.com" && path_name_array[3] === "src") {
        let index;
        if (path_name_array[4] === "branch" && path_name_array[5] !== undefined) {
            index = 5
        }
        else if (path_name_array[4] !== undefined) {
            index = 4
        }
        for (let i = index; i < path_name_array.length; ++i) {
            console.log("el:", path_name_array[i])
            branch += '/' + path_name_array[i]
        }

        document.getElementById("branch").innerText = branch.slice(1)
        let url_str = Url.protocol + Url.host + "/api" + "/" + repository_hostname + "/" + owner + "/" + repository_name + "/src" + branch
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

    // try {
        ok = await preparePage(new URL(document.URL))
        let cloc_promise = await fetch_cloc();
    

    // if (!ok) { return; }
    if (cloc_promise === 202) {
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
        startStreaming(websocket)
    }
    // }
    // catch (err) {
    // if (err instanceof FetchError || err instanceof PrepareError) {
    //     showError(err.status, err.message)
    // } else {
    //     showError(err)
    // }
    // }
}

document.onload = start

async function stopStreaming(ws) {
    return await ws.close()
}

function startStreaming(ws) {
    let send_ping = function () {
        if (ws.readyState === WebSocket.OPEN) {
            console.log("ping ws")
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
        stopRotate()
    }

    ws.onmessage = function (event) {
        if (!interval) { interval = setInterval(rotate, 100) }
        let json = JSON.parse(event.data);
        if (json.Done) {
            let cloc = json.Done;

            console.log("payload", json.Done);
            if (cloc.length > 0) {
                stopStreaming(ws)
                const str = String.fromCharCode(...cloc);
                createTableFromResponse(str);
                document.getElementById("processing").hidden = true
            }
            return
        }
        else if (json.InProgress) {
            let p = json.InProgress;
            // status.innerText = p;
            let lines = p.split(/\r?\n/)
            for (let i = 0; i < lines.length; ++i) {
                let payload = lines[i];
                // console.log("payload line:", payload)
                console.log("Done?", payload, payload.hasOwnProperty("Done"));

                // document.getElementById("status").innerText += payload
                document.getElementById("hint").innerHTML = "Downloading repository into server"
                if (payload.includes("git")) {
                    document.getElementById("git").innerText = payload
                }
                if (payload.includes("Cloning")) {
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
                        console.log("done?", payload);
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
}

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
