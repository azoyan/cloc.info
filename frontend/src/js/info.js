import { gitUrlParse } from "./git_url_parser";
import { createRepositoryIcon, isGithub, buildRepositoryPath, buildRepositoryUrl, documentGetElementById, documentCreateElement, classListRemove, classListAdd, appendChildren, TEXT, DOCUMENT, DIV, parseSccOutput } from "./common";
import {
    TABLE, TABLE_AUTO, FLEX, TRUNCATE, HIDDEN, PY_2, MD, FONT_LIGHT,
    W_FULL,
    TABLE_FIXED,
    TEXT_WHITE, DARK,
    TEXT_XL,
    FONT_MEDIUM,
    UNDERLINE,
    BORDER_T,
    PX_2_5,
    PT_1_5,
    BORDER_B,
    TEXT_START,
    TEXT_END,
    MB_2,
    TEXT_NEUTRAL_700,
    TRANSFORM,
    DARK_TEXT_NEUTRAL_300,
    BORDER_NEUTRAL_300,
    PX_5,
    ROUNDED_LG,
    MY_4,
    PT_2,
    DARK_HOVER_TEXT_WHITE,
    DARK_BORDER_ZINC_500,
    TEXT_BASE,
    MAX_W_SCREEN_LG,
    P_4,
    FLEX_COL,
    HOVER_TEXT_NEUTRAL_800,
    DARK_TEXT_NEUTRAL_200,
    BORDER_NEUTRAL_200,
    ROUNDED_T_LG,
    ROUNDED_R_LG,
    ROUNDED_L_LG,
    ROUNDED_B_LG,
    DARK_TEXT_WHITE,
    MD_TABLE_FIXED
} from './tailwind-classes.js'

let Url = new URL(document.URL);

let json;

const TABLE_DIV = documentGetElementById("t")
const WARNING_DIV = documentGetElementById("warning")
const REPOSITORY_DIV = documentGetElementById("repository")
const COCOMO_DIV = documentGetElementById("cocomo")
const PROCESSING_DIV = documentGetElementById("processing")
const TEXT_DECODER = new TextDecoder()


const SEMICOLON_STR = ":"

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
    console.log("extractContent", response);
    let msg = error_msg ? error_msg + ":\n" : ""
    const contentType = response.headers.get("content-type");
    let result = new Reply(response.status)
    if (contentType && contentType.indexOf("application/json") !== -1) {
        return response.json().then(data => {
            result.setJsonData(data)
            return result
        });
    } else {
        if (response.status >= 400) {
            return response.text().then(text => {
                throw new FetchError(response.status, msg + text)
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

function parseRepositoryPath(url) {
    let path_name_array = url.pathname.split('/').filter(item => item);
    console.log(path_name_array, url)
    if (path_name_array.length < 3) {
        console.error("Incorrect URL:", url)
        throw new PrepareError("Incorrect URL", url + " should contain repository hostname, owner and repository name");
    }

    let repository_hostname = path_name_array[0];
    let owner_index = repository_hostname === "gitflic.ru" && path_name_array[1] === "project" ? 2 : 1;

    if (path_name_array[owner_index] === undefined || path_name_array[owner_index + 1] === undefined) {
        console.error("Incorrect URL:", url)
        throw new PrepareError("Incorrect URL", url + " should contain repository hostname, owner and repository name");
    }

    let owner = path_name_array[owner_index]
    let repository_name = path_name_array[owner_index + 1]

    return {
        repository_hostname,
        owner,
        repository_name,
        remainder: path_name_array.slice(owner_index + 2),
        repository_path: buildRepositoryPath(repository_hostname, owner, repository_name),
    }
}

async function preparePage(url) {
    const { repository_hostname, owner, repository_name, remainder, repository_path } = parseRepositoryPath(url)
    let branch = ""
    const repository_page_url = buildRepositoryUrl(repository_hostname, owner, repository_name.replace(/\.git$/, ''))

    let origin_url = repository_page_url
    let show_url = repository_page_url
    let external_url = repository_page_url
    console.log("origin_url", origin_url)
    let parsed_url = gitUrlParse(origin_url)
    console.log("parsed_url", parsed_url);
    let img = createRepositoryIcon(repository_hostname, 32, 32)

    // if (repository_name.slice(-4) === ".git") { repository_name = repository_name.slice(0, -4) }
    // else if (repository_name.slice(-4) !== ".git") { origin_url += ".git" }

    let pic_ref = '<a target="_blank" rel="noopener noreferrer canonical" href="' + origin_url + '">' + img + '</a>'
    // documentGetElementById("url_pic").innerHTML = pic_ref
    // }
    // catch (e) {
    //     throw new Error("Can't setup URL" + e)
    // }

    const api_base = Url.origin + "/api/" + repository_path

    if (remainder[0] === undefined) {
        let branch_info = await fetch_branch_info(api_base)
        branch = branch_info.getJsonData()
        documentGetElementById("branch").innerText = branch.default_branch
        let commit_info = await fetch_branch_commit(api_base + "/tree/" + branch.default_branch)
        let commit = commit_info.getJsonData()

        documentGetElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname === "github.com" && remainder[0] === "tree" && remainder[1] !== undefined) {
        for (let i = 1; i < remainder.length; ++i) {
            console.log("el:", remainder[i])
            branch += '/' + remainder[i]
        }
        show_url = repository_page_url + "/tree" + branch
        external_url = show_url
        documentGetElementById("branch").innerText = branch.slice(1)
        let url_str = api_base + "/tree" + branch
        let commit = await fetch_branch_commit(url_str)
        documentGetElementById("commit").innerText = commit.commit
    }
    else if (isGithub(repository_hostname)) {
        if (remainder[0] === "tree" && remainder[1] !== undefined) {
            for (let i = 1; i < remainder.length; ++i) {
                console.log("el:", remainder[i])
                branch += '/' + remainder[i]
            }
        }
        else if (remainder[0] === "-" && remainder[1] === "tree" && remainder[2] !== undefined) {
            for (let i = 2; i < remainder.length; ++i) {
                console.log("el:", remainder[i])
                branch += '/' + remainder[i]
            }
        }
        else {
            let error_msg = url + "\nAfter tree/ must be followed by a branch name"
            console.error(error_msg)
            throw new PrepareError("Incorrect URL", error_msg)
        }
        show_url = remainder[0] === "-" ? repository_page_url + "/-/tree" + branch : repository_page_url + "/tree" + branch
        external_url = show_url
        documentGetElementById("branch").innerText = branch.slice(1)
        let url_str = api_base + "/tree" + branch
        let commit = await fetch_branch_commit(url_str)
        documentGetElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname === "gitflic.ru" && remainder[0] === "tree" && remainder[1] !== undefined) {
        for (let i = 1; i < remainder.length; ++i) {
            console.log("el:", remainder[i])
            branch += '/' + remainder[i]
        }
        show_url = repository_page_url + "/file/?branch=" + encodeURIComponent(branch.slice(1))
        external_url = show_url
        documentGetElementById("branch").innerText = branch.slice(1)
        let url_str = api_base + "/tree" + branch
        let commit = await fetch_branch_commit(url_str)
        documentGetElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname === "bitbucket.org" && remainder[0] === "src" && remainder[1] !== undefined) {
        for (let i = 1; i < remainder.length; ++i) {
            console.log("el:", remainder[i])
            branch += '/' + remainder[i]
        }
        show_url = repository_page_url + "/src" + branch
        external_url = show_url
        documentGetElementById("branch").innerText = branch.slice(1)
        let url_str = api_base + "/src" + branch
        let commit = await fetch_branch_commit(url_str)
        documentGetElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname == "codeberg.org" && remainder[0] === "src") {
        let index;
        if (remainder[1] === "branch" && remainder[2] !== undefined) {
            index = 2
        }
        else if (remainder[1] !== undefined) {
            index = 1
        }
        for (let i = index; i < remainder.length; ++i) {
            console.log("el:", remainder[i])
            branch += '/' + remainder[i]
        }

        show_url = remainder[1] === "branch" ? repository_page_url + "/src/branch" + branch : repository_page_url + "/src" + branch
        external_url = show_url
        documentGetElementById("branch").innerText = branch.slice(1)
        let url_str = api_base + "/src" + branch
        let commit = await fetch_branch_commit(url_str)
        documentGetElementById("commit").innerText = commit.commit
    }
    else if (repository_hostname == "gitea.com" && remainder[0] === "src") {
        let index;
        if (remainder[1] === "branch" && remainder[2] !== undefined) {
            index = 2
        }
        else if (remainder[1] !== undefined) {
            index = 1
        }
        for (let i = index; i < remainder.length; ++i) {
            console.log("el:", remainder[i])
            branch += '/' + remainder[i]
        }

        show_url = remainder[1] === "branch" ? repository_page_url + "/src/branch" + branch : repository_page_url + "/src" + branch
        external_url = show_url
        documentGetElementById("branch").innerText = branch.slice(1)
        let url_str = api_base + "/src" + branch
        let commit = await fetch_branch_commit(url_str)
        documentGetElementById("commit").innerText = commit.commit
    }
    else {
        let error_msg = url + "\nAfter tree/ must be followed by a branch name"
        console.error(error_msg)
        throw new PrepareError("Incorrect URL", error_msg)
    }

    documentGetElementById("url").innerText = show_url
    documentGetElementById("url").setAttribute("href", external_url)

    return true
}

function showError(status, message) {
    const bodyText = message
        ? message
        : status instanceof Error
            ? status.message
            : String(status || "Unexpected error while loading repository information.");
    const headerText = typeof status === "number"
        ? `HTTP ${status}`
        : status && !(status instanceof Error)
            ? String(status)
            : "Request error";

    WARNING_DIV.replaceChildren();

    const heading = documentCreateElement("p")
    classListAdd(heading, FONT_MEDIUM, MB_2)
    heading.innerText = headerText

    const text = documentCreateElement("p")
    text.innerText = bodyText

    appendChildren(WARNING_DIV, heading, text)
    classListRemove(WARNING_DIV, HIDDEN)
    classListAdd(PROCESSING_DIV, HIDDEN)
    classListAdd(TABLE_DIV, HIDDEN)
    classListAdd(COCOMO_DIV, HIDDEN)
}

function decodeBytes(bytes) {
    return TEXT_DECODER.decode(new Uint8Array(bytes))
}

function createWarning(prev) {
    let date = new Date(prev.date).toISOString().substring(0, 16).replace('T', ' ').replace(' ', "\xa0");

    let p1 = documentCreateElement("p")
    let text = documentCreateElement(TEXT);
    text.innerText = "This information is current as of "

    let strongDate = documentCreateElement(TEXT)
    classListAdd(strongDate, FONT_MEDIUM)
    strongDate.innerText = date

    let text2 = documentCreateElement("span")
    text2.innerText = " (Commit: "
    let strongCommit = documentCreateElement("span")

    strongCommit.innerText = prev.commit + ")"
    classListAdd(text2, TRUNCATE)

    appendChildren(p1, text, strongDate, text2, strongCommit)
    return p1

}

async function start(_e) {
    let ok = false

    try {
        ok = await preparePage(new URL(DOCUMENT.URL))
        let cloc_reply = await fetch_cloc();

        console.log("cloc_promise", cloc_reply)

        if (cloc_reply.statusCode === 200) {
            createTableFromResponse(cloc_reply.getTextData())
            classListAdd(TABLE_DIV, BORDER_T)
            classListRemove(TABLE_DIV, HIDDEN)
            return
        }
        else if (cloc_reply.statusCode === 206) {
            let prev = cloc_reply.getJsonData().Previous;
            let data = decodeBytes(prev.data);
            classListRemove(REPOSITORY_DIV, HIDDEN)
            appendChildren(WARNING_DIV, createWarning(prev, prev))
            classListRemove(TABLE_DIV, BORDER_T, ROUNDED_L_LG, ROUNDED_R_LG, ROUNDED_T_LG, HIDDEN)
            classListRemove(WARNING_DIV, ROUNDED_B_LG, HIDDEN, BORDER_T, ROUNDED_R_LG, ROUNDED_L_LG)
            createTableFromResponse(data)
        }
        classListRemove(PROCESSING_DIV, HIDDEN)

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
        // documentGetElementById("repository").hidden = true
        // documentGetElementById("processing").hidden = true
    }
}

DOCUMENT.addEventListener("DOMContentLoaded", start)

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
        // console.log("open ws", event);
        setInterval(send_ping, 500);
        let message = { start: true }
        worker.postMessage(message)
    }

    ws.onclose = function (event) {
        console.log("event", event);
        documentGetElementById("hint").setAttribute(HIDDEN, true)
        console.log("WEB SOCKET  CLOSED");
        stopRotate()
    }

    ws.onmessage = function (event) {
        json = JSON.parse(event.data);
        if (json.Done) {
            let cloc = json.Done;

            // console.log("payload", json.Done);
            if (cloc.length > 0) {
                stopStreaming(ws)
                const CLOC = decodeBytes(cloc);
                createTableFromResponse(CLOC);
                classListAdd(PROCESSING_DIV, HIDDEN)
                classListAdd(WARNING_DIV, HIDDEN)
                classListAdd(TABLE_DIV, BORDER_T)
                classListRemove(TABLE_DIV, HIDDEN)
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
                classListAdd(documentGetElementById("status"), PY_2)

                if (payload.includes("git")) {
                    documentGetElementById("git").innerText = payload
                }
                if (payload.includes("Cloning")) {
                    documentGetElementById("cloning").innerText = payload
                }
                if (payload.includes("Enumerating")) {
                    let parts = payload.split(SEMICOLON_STR);
                    if (parts.length >= 3) {

                        // console.log(percent)
                        // documentGetElementById("pg_enumerating").style.width = percent;
                        documentGetElementById("enumerating").innerText = "remote: Enumerating objects:" + parts[parts.length - 1]
                    }
                }
                if (payload.includes("Counting")) {
                    let parts = payload.split(SEMICOLON_STR);
                    if (parts.length >= 3) {
                        let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g)[0])
                        // console.log("counting", percent)
                        percent = percent * 1 / 100
                        documentGetElementById("pg_counting").style.width = percent + '%';
                        documentGetElementById("counting").innerText = "remote: Counting objects:" + parts[parts.length - 1]

                    }
                }
                if (payload.includes("Compressing")) {
                    let parts = payload.split(SEMICOLON_STR);
                    if (parts.length >= 3) {
                        let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g))
                        // console.log("compressing", percent)
                        percent = percent * 16 / 100
                        documentGetElementById("pg_compressing").style.width = percent + '%';
                        documentGetElementById("compressing").innerText = "remote: Compressing objects:" + parts[parts.length - 1]
                    }
                }
                if (payload.includes("Total")) {
                    let parts = payload.split(SEMICOLON_STR);
                    if (parts.length >= 2) {
                        documentGetElementById("total").innerText = "remote:" + parts[parts.length - 1]
                    }
                }
                if (payload.includes("Receiving")) {
                    let parts = payload.split(SEMICOLON_STR);
                    if (parts.length >= 2) {
                        let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g)[0]);
                        percent = percent * 75 / 100
                        documentGetElementById("pg_receiving").style.width = percent + '%';
                        documentGetElementById("receiving").innerText = "Receiving objects:" + parts[parts.length - 1];
                    }
                }
                if (payload.includes("Resolving")) {
                    let parts = payload.split(SEMICOLON_STR);
                    if (parts.length >= 2) {
                        let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g))
                        percent = percent * 4 / 100
                        documentGetElementById("pg_resolving").style.width = percent + '%';
                        documentGetElementById("resolving").innerText = "Resolving deltas:" + parts[parts.length - 1]
                    }
                }
                if (payload.includes("Updating")) {
                    if (payload.includes("done")) {
                        console.log("done?", payload);
                        documentGetElementById("hint").innerText = "Counting lines of code"
                    }
                    let parts = payload.split(SEMICOLON_STR);
                    if (parts.length >= 2) {
                        let percent = parseInt(parts[parts.length - 1].match(/[0-99]+/g))
                        percent = percent * 9 / 100
                        documentGetElementById("pg_updating").style.width = percent + '%';
                        documentGetElementById("updating").innerText = "Updating objects:" + parts[parts.length - 1]
                    }
                }
            }
        }
    }
}


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

function createTableFromResponse(data) {
    const parsed = parseSccOutput(data)
    if (parsed === null) {
        return
    }

    let table = documentCreateElement(TABLE)
    classListAdd(table, TABLE, TABLE_AUTO, MD_TABLE_FIXED, W_FULL, DARK_TEXT_WHITE)

    let thead = createTableHead(parsed.header);
    let tbody = documentCreateElement("tbody")

    for (let i = 0; i < parsed.rows.length; ++i) {
        let row = createTableRow(parsed.rows[i])
        appendChildren(tbody, row)
    }
    appendChildren(table, thead, tbody)

    TABLE_DIV.replaceChildren()

    let caption = documentCreateElement(DIV);
    caption.textContent = parsed.processed;
    classListAdd(caption, PT_1_5, PX_2_5, FONT_LIGHT, TEXT_NEUTRAL_700, DARK_TEXT_NEUTRAL_300)

    appendChildren(TABLE_DIV, table, caption)

    createCocomoFromResponse(parsed.cocomo)
}

function createTableHead(array) {
    let thead = documentCreateElement('thead')
    let tr = documentCreateElement('tr')

    array.forEach((item, index) => {
        let th = documentCreateElement('th');
        th.scope = 'col'
        if (index === 0) {
            classListAdd(th, TEXT_START)
        } else {
            classListAdd(th, TEXT_END)
        }
        classListAdd(th, PX_2_5, P_4, FONT_MEDIUM, BORDER_B, TEXT_NEUTRAL_700, DARK_TEXT_NEUTRAL_300, BORDER_NEUTRAL_300)
        th.textContent = item;

        appendChildren(tr, th)
    });

    appendChildren(thead, tr)
    return thead;
}

function createTableRow(array) {
    let row = documentCreateElement('tr');

    array.forEach((item, index) => {
        let td = documentCreateElement('td');
        classListAdd(td, PX_2_5, PY_2, BORDER_B, TEXT_NEUTRAL_700, DARK_TEXT_NEUTRAL_200, BORDER_NEUTRAL_200, DARK_BORDER_ZINC_500)

        if (index === 0) {
            td.textContent = item.substring(0, 17)
            if (item.length >= 20) {
                td.textContent += '...'
            }
            td.title = item
            classListAdd(td, FONT_MEDIUM, TRUNCATE)
        } else {
            classListAdd(td, TEXT_END)
            const num = Number(item);
            if (!isNaN(num)) {
                td.textContent = num.toLocaleString()
            } else {
                td.textContent = item
            }
        }
        appendChildren(row, td)
    });

    return row;
}

function createCocomoFromResponse(cocomo_data) {
    COCOMO_DIV.replaceChildren()
    classListRemove(COCOMO_DIV, HIDDEN)
    let card = documentCreateElement("ul")
    classListAdd(card, MAX_W_SCREEN_LG, MY_4, FLEX, FLEX_COL, ROUNDED_LG, DARK_TEXT_NEUTRAL_200)

    let cardTitle = documentCreateElement(DIV)


    cardTitle.textContent = 'COCOMO'

    let cardSubtitle = documentCreateElement(DIV)
    classListAdd(cardSubtitle, MB_2, TEXT_NEUTRAL_700, FONT_LIGHT, DARK_TEXT_NEUTRAL_300, TEXT_BASE)
    cardSubtitle.textContent = 'Constructive Cost Model ('
    let link = documentCreateElement('a')
    link.target = '_blank'
    link.rel = 'noopener noreferrer canonical'
    link.href = 'https://en.wikipedia.org/wiki/COCOMO'
    link.textContent = 'wiki';
    classListAdd(link, FONT_MEDIUM, UNDERLINE, HOVER_TEXT_NEUTRAL_800, DARK_HOVER_TEXT_WHITE)
    classListAdd(cardTitle, TEXT_XL, FONT_MEDIUM, PX_5, BORDER_B, DARK_BORDER_ZINC_500)

    appendChildren(cardSubtitle, link, document.createTextNode(')'))
    appendChildren(cardTitle, cardSubtitle)
    appendChildren(card, cardTitle)

    for (let i = 0; i < cocomo_data.length; i++) {
        let paragraph = documentCreateElement('li')
        classListAdd(paragraph, PX_5, PT_2)
        paragraph.textContent = cocomo_data[i]

        appendChildren(card, paragraph)
    }
    appendChildren(COCOMO_DIV, card)
}

const HEADING_ONE = documentGetElementById('headingOne');

HEADING_ONE.addEventListener('click', () => {
    documentGetElementById('collapseOne').classList.toggle('hidden');
    const icon = HEADING_ONE.querySelector('svg');
    icon.classList.toggle('rotate-180');
});

const worker = new Worker("/assets/sw.js")
worker.onmessage = (event) => rotate(event.data);

// Get the favicon element
const FAVICON_ELEMENT = documentGetElementById('favicon');
const ORIGINAL_FAVICON_HREF = FAVICON_ELEMENT?.href ?? FAVICON_ELEMENT?.getAttribute('href') ?? '';

// Load the original favicon image
const originalFavicon = new Image();
originalFavicon.src = ORIGINAL_FAVICON_HREF;

const LOGO = documentGetElementById("logo")

function rotate(angle) {
    LOGO.setAttribute(TRANSFORM, "rotate(" + angle + ")");

    if (!FAVICON_ELEMENT || !canRotateImage(originalFavicon)) {
        return
    }

    const rotatedFavicon = rotateImage(originalFavicon, angle);
    if (rotatedFavicon !== null) {
        FAVICON_ELEMENT.href = rotatedFavicon;
    }
}

function rotateImage(image, angle) {
    if (!canRotateImage(image)) {
        return null
    }

    const canvas = documentCreateElement('canvas');
    const ctx = canvas.getContext('2d');
    if (ctx === null) {
        return null
    }

    canvas.width = image.width;
    canvas.height = image.height;

    ctx.translate(canvas.width / 2, canvas.height / 2);
    ctx.rotate((Math.PI / 180) * angle);
    ctx.drawImage(image, -canvas.width / 2, -canvas.height / 2);

    return canvas.toDataURL('image/x-icon');
}

function canRotateImage(image) {
    return image.complete && image.naturalWidth > 0 && image.naturalHeight > 0
}

function stopRotate() {
    worker.postMessage({ start: false })
    rotate(0);
    if (FAVICON_ELEMENT) {
        FAVICON_ELEMENT.href = ORIGINAL_FAVICON_HREF;
    }
    LOGO.removeAttribute(TRANSFORM)
}
