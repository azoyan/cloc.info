let Url = new URL(document.URL);

let json;


async function fetch_cloc() {
    let response = await fetch(Url, { headers: { 'If-Match': 'cloc' } });
    return extractContent(response, "Error at fetching lines of code");
}
class Reply {
    constructor(statusCode) {
        this.statusCode = statusCode;
        this.jsonData = null;
        this.textData = null;
    }

    getStatusCode() {
        return this.statusCode;
    }

    setStatusCode(statusCode) {
        this.statusCode = statusCode;
    }

    getJsonData() {
        return this.jsonData;
    }

    setJsonData(jsonData) {
        this.jsonData = jsonData;
    }

    getTextData() {
        return this.textData;
    }

    setTextData(textData) {
        this.textData = textData;
    }
}
function extractContent(response, error_msg) {
    message = error_msg ? error_msg + ":\n" : ""
    const contentType = response.headers.get("content-type");
    let result = new Reply(response.status)
    if (contentType && contentType.indexOf("application/json") !== -1) {
        return response.json().then(data => {
            result.setJsonData(data)
            return result
        });
    } else {
        if (response.status >= 400) {
            response.text().then(text => {
                throw new FetchError(response.status, message + text)
            });
        }
        else {
            return response.text().then(text => {
                result.setTextData(text)
                return result;
            });
        }
    }
}

async function fetch_branch_info(url_str) {
    let branch_api_info_url = new URL(url_str)
    console.log("fetch branch_api_info_url", branch_api_info_url)
    let response_branch_info = await fetch(branch_api_info_url);
    return extractContent(response_branch_info, "Error at fetching default branch")
}

async function fetch_branch_commit(url_str) {
    let branch_commit = new URL(url_str)
    console.log("fetch branch_commit", branch_commit)
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
    for (let i = 3; i < path_name_array.length; ++i) { show_url += '/' + path_name_array[i] }
    document.getElementById("url").innerText = show_url
    document.getElementById("url").setAttribute("href", origin_url)
    document.getElementById("url_pic").innerHTML = pic_ref
    // }
    // catch (e) {
    //     throw new Error("Can't setup URL" + e)
    // }

    if (path_name_array[3] === undefined) {
        let branch_info = await fetch_branch_info(Url.protocol + Url.host + "/api" + Url.pathname)
        branch = branch_info.getJsonData()
        document.getElementById("branch").innerText = branch.default_branch
        let commit_info = await fetch_branch_commit(Url.protocol + Url.host + "/api" + Url.pathname + "/tree/" + branch.default_branch)
        let commit = commit_info.getJsonData()
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
    let alert = document.getElementById("alert_block")
    alert.classList.toggle('show')
    let bodyText = message ? message : ""
    let headerText = status ? status : ""
    alert.appendChild(createAlertBlock("danger", headerText, bodyText))
}

function showWarning(message) {
    let alert = document.getElementById("warning")
    alert.classList.toggle("collapse")
    let bodyText = message ? message : ""
    let headerText = ""
    alert.appendChild(createAlertBlock("warning", headerText, bodyText))
}

function createAlertBlock(alertType, headerText, bodyText) {
    let alert = document.createElement("div")
    alert.classList.add("alert", `alert-${alertType}`)
    alert.setAttribute("role", "alert")

    let header = document.createElement("h")
    header.classList.add("alert-heading")
    header.innerText = headerText
    alert.appendChild(header)

    let body = document.createElement("p")
    body.classList.add("mb-4", "font-monospace", "text-break")
    body.innerText = bodyText
    alert.appendChild(body)

    return alert
}

async function start(_e) {
    let ok = false

    try {
        ok = await preparePage(new URL(document.URL))
        let cloc_reply = await fetch_cloc();

        console.log("cloc_promise", cloc_reply)

        if (cloc_reply.statusCode === 200) {
            createTableFromResponse(cloc_reply.getTextData());
            return
        }
        else if (cloc_reply.statusCode === 206) {
            let prev = cloc_reply.getJsonData().Previous;
            let data = String.fromCharCode(...prev.data);

            showWarning(warningText(prev.date, prev.commit))
            createTableFromResponse(data);
        }
        document.getElementById("processing").removeAttribute("hidden")
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
    catch (err) {
        if (err instanceof FetchError || err instanceof PrepareError) {
            showError(err.status, err.message)
        } else {
            showError(err)
        }
        document.getElementById("repository").hidden = true
        document.getElementById("processing").hidden = true
    }
}

document.onload = start

async function stopStreaming(ws) {
    return await ws.close()
}

function startStreaming(ws) {
    // setFavicon('/static/animated.gif')
    let send_ping = function () {
        if (ws.readyState === WebSocket.OPEN) {
            // console.log("ping ws")
            ws.send("ping")
        }
    }

    ws.onopen = function (event) {
        // console.log("open ws", event);
        setInterval(send_ping, 500);
        let message = { start: true }
        worker.postMessage(message)
    }

    ws.onclose = function (event) {
        console.log("event", event);
        document.getElementById("hint").setAttribute("hidden", true)
        console.log("WEB SOCKET  CLOSED");
        stopRotate()
        // setFavicon('/static/favicon.ico')
    }

    ws.onmessage = function (event) {
        json = JSON.parse(event.data);
        if (json.Done) {
            let cloc = json.Done;

            // console.log("payload", json.Done);
            if (cloc.length > 0) {
                stopStreaming(ws)
                const CLOC = String.fromCharCode(...cloc);
                createTableFromResponse(CLOC);
                document.getElementById("processing").hidden = true
                document.getElementById("warning").classList.toggle('collapse')
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
                // console.log("Done?", payload, payload.hasOwnProperty("Done"));

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

function warningText(dateStr, commit) {
    let date = new Date(dateStr).toString()
    return `The information about the repository provided below is accurate as of ${date} and applies to commit ${commit}.`
}

let interval;
const worker = new Worker("/static/sw.js")
worker.onmessage = (event) => rotate(event.data);

// Get the favicon element
const faviconElement = document.getElementById('favicon');

// Load the original favicon image
const originalFavicon = new Image();
originalFavicon.src = '/static/favicon.ico';

const LOGO = document.getElementById("logo")

function rotate(angle) {
    // LOGO.setAttribute("transform", "rotate(" + angle + ")");

    const rotatedFavicon = rotateImage(originalFavicon, angle);
    faviconElement.href = rotatedFavicon;
}

function rotateImage(image, angle) {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    canvas.width = image.width;
    canvas.height = image.height;

    ctx.translate(canvas.width / 2, canvas.height / 2);
    ctx.rotate((Math.PI / 180) * angle);
    ctx.drawImage(image, -canvas.width / 2, -canvas.height / 2);

    return canvas.toDataURL('image/x-icon');
}

function stopRotate() {
    worker.postMessage({ start: false })
    rotate(0);
    faviconElement.href = originalFavicon.src;
    LOGO.removeAttribute("transform")
}
