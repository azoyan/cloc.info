
class Api {
    constructor() { }

    recent(response) {
        let element = document.getElementById(`recent`)
        element.innerHTML = ''

        response.sort(sort_recent)
        for (let i = 0; i < response.length; ++i) {
            let item = new ListItem(response[i]).recent()
            element.appendChild(item)
        }
    }

    popular(response) {
        let element = document.getElementById(`popular`)
        element.innerHTML = ''

        response.sort(sort_popular)
        for (let i = 0; i < response.length; ++i) {
            let item = new ListItem(response[i]).popular()
            element.appendChild(item)
        }
    }

    largest(response) {
        let element = document.getElementById(`largest`)
        element.innerHTML = ''

        response.sort(sort_largest)
        for (let i = 0; i < response.length; ++i) {
            let item = new ListItem(response[i]).largest()
            element.appendChild(item)
        }
    }
}

function sort_recent(a, b) {
    if (a.time > b.time) return -1;
    if (a.time < b.time) return 1;
    return 0
}
function sort_popular(a, b) {
    if (a.count > b.count) return -1;
    if (a.count < b.count) return 1;
    return 0
}
function sort_largest(a, b) {
    if (a.size > b.size) return -1;
    if (a.size < b.size) return 1;
    return 0
}

function start() {
    fetchApi("recent")
    fetchApi("popular")
    fetchApi("largest")
}

const API = new Api();

async function fetchApi(apiName) {
    let Url = new URL(document.URL);
    let url = new URL(Url.protocol + Url.host + `/api/${apiName}/15`);


    fetch(url).then((response) => response.json()).then((response) => API[apiName](response))
        .catch(function (e) {
            console.log(e)
        })
}

function createExternalLink(icon) {
    let buttonGroup = document.createElement("button")

    buttonGroup.classList.add("btn", "btn-sm", "btn-outline-dark")
    buttonGroup.appendChild(icon)

    return buttonGroup
}

function createExternalLinks(repository) {
    let icon;
    if (repository.hostname === "github.com") {
        let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="18.75" height="18.75" fill="currentColor" class="bi bi-github" viewBox="-0.5 -1.5 17.5 17.5"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>`
        icon = new DOMParser().parseFromString(svg, "text/xml").firstChild

    }
    else if (repository.hostname === "gitlab.com") {
        icon = document.createElement("img")
        icon.setAttribute("width", "18px")
        icon.setAttribute("height", "22px")
        icon.setAttribute("src", "/static/gitlab-3.svg")
        icon.classList.add("float-start")
    }
    else if (repository.hostname === "bitbucket.org") {
        icon = document.createElement("img")
        icon.setAttribute("width", "20px")
        icon.setAttribute("height", "22px")
        icon.setAttribute("src", "/static/bitbucket-icon.svg")
        icon.classList.add("float-start")
    }
    else if (repository.hostname === "codeberg.org") {
        let svg = `<svg style="color: #2185d0" role="img" width="18px" height="18px" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Codeberg</title>
                <path d="M11.955.49A12 12 0 0 0 0 12.49a12 12 0 0 0 1.832 6.373L11.838 5.928a.187.14 0 0 1 .324 0l10.006 12.935A12 12 0 0 0 24 12.49a12 12 0 0 0-12-12 12 12 0 0 0-.045 0zm.375 6.467l4.416 16.553a12 12 0 0 0 5.137-4.213z" fill="#2185d0"></path>
            </svg>`
        icon = new DOMParser().parseFromString(svg, "text/xml").firstChild
    }
    else if (repository.hostname === "gitea.com") {
        icon = document.createElement("img")
        icon.setAttribute("width", "18px")
        icon.setAttribute("height", "22px")
        icon.setAttribute("src", "/static/gitea.svg")
        icon.classList.add("float-start")
    }
    else {
        let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="18px" height="18px" style="color: #f34f29;" fill="currentColor" class="bi bi-git" viewBox="0 -1 17 17">
        <path d="M15.698 7.287 8.712.302a1.03 1.03 0 0 0-1.457 0l-1.45 1.45 1.84 1.84a1.223 1.223 0 0 1 1.55 1.56l1.773 1.774a1.224 1.224 0 0 1 1.267 2.025 1.226 1.226 0 0 1-2.002-1.334L8.58 5.963v4.353a1.226 1.226 0 1 1-1.008-.036V5.887a1.226 1.226 0 0 1-.666-1.608L5.093 2.465l-4.79 4.79a1.03 1.03 0 0 0 0 1.457l6.986 6.986a1.03 1.03 0 0 0 1.457 0l6.953-6.953a1.031 1.031 0 0 0 0-1.457"/>
        </svg>`
        icon = new DOMParser().parseFromString(svg, "text/xml").firstChild
    }
    return createExternalLink(icon)
}

class ListItem {
    constructor(repository) {
        this.repository = repository
        this.id = ""
        this.description = null
        this.collapse = null
        if (this.repository.repository_name.slice(-4) === ".git") {
            this.repository.repository_name = this.repository.repository_name.slice(0, -4)
        }
    }

    popular() {
        let repository = this.repository;
        this.id = repository.repository_name + "-popular"
        let repository_array = repository.branches;
        let totalCount = 0;
        for (let i = 0; i < repository_array.length; ++i) { totalCount += repository_array[i].count }

        if (repository_array.length > 1) {
            this.collapse = new CollapseContent(this.id, repository).popular()
        }

        this.description = createSmallText(createViewText(totalCount))
        return this.toElement()
    }

    recent() {
        let repository = this.repository;
        this.id = repository.repository_name + "-recent"

        let now = Date.now()
        let date = Date.parse(repository.time)
        let diff = delta_time(now, date)

        let repository_array = repository.branches

        if (repository_array.length > 1) {
            this.collapse = new CollapseContent(this.id, repository).recent()
        }
        this.description = createSmallText(diff)

        return this.toElement()
    }

    largest() {
        let repository = this.repository;
        this.id = this.repository.repository_name + "-largest"
        let bytes = formatBytes(repository.size)

        let repository_array = repository.branches;
        if (repository_array.length > 1) {
            this.collapse = new CollapseContent(this.id, repository).largest()
        }
        this.description = createSmallText(bytes)

        return this.toElement()
    }

    toElement() {
        let repository = this.repository
        let repository_array = repository.branches;
        let listItem = document.createElement("li");
        listItem.classList.add("list-group-item")

        let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name

        let row = createRow("align-items-center");
        let col1 = createColumn("col-sm", "text-truncate")

        let a = document.createElement("a")
        a.setAttribute("target", "_blank")
        a.setAttribute("rel", "noopener noreferrer canonical")
        a.setAttribute("href", local_href)
        let title = `${repository.repository_name}`
        a.setAttribute("title", title)

        a.classList.add("link-dark", "me-2")
        a.innerText = repository.repository_name
        col1.appendChild(a)
        row.appendChild(col1)

        let col2 = createColumn("col-auto")
        col2.appendChild(this.description)
        row.appendChild(col2)

        let col3 = document.createElement("a")
        col3.setAttribute("target", "_blank")
        col3.setAttribute("rel", "noopener noreferrer canonical")
        let href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
        col3.setAttribute("href", href)
        col3.setAttribute("title", `Open repository ${href}`)
        col3.classList.add("col", "col-auto")
        let externalLink = createExternalLinks(repository);
        col3.appendChild(externalLink)

        row.appendChild(col3)

        let col4 = createColumn("col-1")
        row.appendChild(col4)
        if (repository_array.length > 1) {
            let button = createCollapseButton(this.id)
            col4.appendChild(button)

            let collapse = this.collapse

            listItem.appendChild(row)
            listItem.appendChild(collapse)
        }
        else {
            listItem.appendChild(row)
        }

        return listItem
    }
}

function createLargestItem(repository) {
    let repository_array = repository.branches[0]

    let listItem = document.createElement("li");
    listItem.classList.add("list-group-item")
    if (repository.repository_name.slice(-4) === ".git") {
        repository.repository_name = repository.repository_name.slice(0, -4)
    }
    let id = repository.repository_name + "-largest"
    let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name

    let row = createRow();
    let col1 = createColumn("text-truncate")

    let a = document.createElement("a")
    a.setAttribute("target", "_blank")
    a.setAttribute("rel", "noopener noreferrer canonical")
    a.setAttribute("href", local_href)
    a.classList.add("link-dark", "me-2")
    a.innerText = repository.repository_name
    col1.appendChild(a)
    row.appendChild(col1)

    let totalCount = 0;
    for (let i = 0; i < repository_array.length; ++i) { totalCount += repository_array[i].count }

    let bytes = formatBytes(repository.size)
    let smallText = createSmallText(`${bytes}`)

    col1.appendChild(smallText)

    let col3 = document.createElement("a")
    col3.setAttribute("target", "_blank")
    col3.setAttribute("rel", "noopener noreferrer canonical")
    let href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    col3.setAttribute("href", href)
    col3.classList.add("col", "col-auto")
    col3.appendChild(createExternalLinks(repository))

    row.appendChild(col3)

    let col4 = createColumn("col-1")
    row.appendChild(col4)
    if (repository_array.length > 1) {
        let button = createCollapseButton(id)
        col4.appendChild(button)

        let collapse = new CollapseContent(id, repository).largest();

        listItem.appendChild(row)
        listItem.appendChild(collapse)
    } else {
        listItem.appendChild(row)
    }

    return listItem
}

function createColumn(...tokens) {
    let col = document.createElement("div")
    col.classList.add("col")
    for (let i = 0; i < tokens.length; i++) {
        col.classList.add(tokens[i])
    }
    return col
}

function createRow(...tokens) {
    let row = document.createElement("div")
    row.classList.add("row")
    if (tokens.length > 0) {
        row.classList.add(tokens)
    }
    return row
}
class CollapseContent {
    constructor(id, repository) {
        this.id = id;
        this.repository_array = repository.branches
        this.repository = repository;
        this.elements = [];
    }
    recent() {
        this.repository_array.sort(sort_recent)
        for (let i = 0; i < this.repository_array.length; ++i) {
            let time = this.repository_array[i].time;
            let now = Date.now()
            let date = Date.parse(time)
            let diff = delta_time(now, date)
            let small = createSmallText(diff)
            this.elements[i] = small
        }
        return this.toElement()
    }
    largest() {
        this.repository_array.sort(sort_largest)
        for (let i = 0; i < this.repository_array.length; ++i) {
            let size = this.repository_array[i].size;
            let bytes = formatBytes(size)
            let small = createSmallText(bytes)
            this.elements[i] = small
        }
        return this.toElement();
    }

    popular() {
        this.repository_array.sort(sort_popular)
        for (let i = 0; i < this.repository_array.length; ++i) {
            let count = this.repository_array[i].count;
            let text = createViewText(count)
            let small = createSmallText(text)
            this.elements[i] = small
        }
        return this.toElement()
    }

    toElement() {
        let div = document.createElement("div");
        div.id = this.id;
        div.classList.add("collapse", "row", "mt-3");

        for (let i = 0; i < this.repository_array.length; ++i) {
            let row = createRow();

            let col1 = createColumn("col-sm", "text-truncate");

            let a = document.createElement("text");
            let branch = this.repository_array[i].branch_name;
            a.innerText = branch
            a.setAttribute("title", branch)

            col1.appendChild(a);
            row.appendChild(col1);

            let col2 = createColumn("col-auto", "text-truncate");
            col2.appendChild(this.elements[i]);
            row.appendChild(col2);

            // let col3 = createColumn("col-sm");
            // row.appendChild(col3)
            div.appendChild(row);
        }

        return div;
    }
}

function createSmallText(text) {
    let div = document.createElement("small")
    div.classList.add("fw-light")
    div.innerText = text
    return div
}

function createCollapseButton(id) {
    let button = document.createElement("div")
    // <button class="" type="button" data-bs-toggle="collapse" data-bs-target="#collapseOne" aria-expanded="true" aria-controls="collapseOne">
    // button.classList.add("btn", "btn-primary")
    button.setAttribute("type", "button")
    button.setAttribute("data-bs-toggle", "collapse")
    button.setAttribute("data-bs-target", `#${id}`)
    button.setAttribute("aria-expanded", "false")
    button.setAttribute("aria-controls", id)

    let icon = document.createElement("i")
    icon.classList.add("text-secondary", "bi", "bi-chevron-expand")

    button.appendChild(icon)
    return button
}

function delta_time(now, date) {
    let dt = (now - date) / 1000
    if (dt > 60 && dt < 7200) {
        dt = Math.round(dt / 60) + " minutes ago"
    }
    else if (dt > 7200 && dt < 86400) {
        dt = Math.round(dt / 3600) + " hours ago"
    }
    else if (dt > 86400) {
        dt = Math.round(dt / 86400) + " days ago"
    }

    else {
        dt = Math.round(dt) + " seconds ago"
    }
    return dt
}
function createViewText(count) { return count > 1 ? `${count} views` : `${count} view` }
function formatBytes(a, b = 2, k = 1024) { with (Math) { let d = floor(log(a) / log(k)); return 0 == a ? "0 Bytes" : parseFloat((a / pow(k, d)).toFixed(max(0, b))) + " " + ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"][d] } }

document.addEventListener("DOMContentLoaded", start);
