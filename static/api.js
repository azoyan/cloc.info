
function start() {
    let api = new Api()

    api.recent().then(data => {
        let fragment = new RecentList(data).toDocumentFragment()
        document.getElementById("recent").appendChild(fragment)
    })
    api.popular().then(data => {
        let fragment = new PopularList(data).toDocumentFragment()
        document.getElementById("popular").appendChild(fragment)
    })


    api.largest().then(data => {
        let fragment = new LargestList(data).toDocumentFragment()
        document.getElementById("largest").appendChild(fragment)
    })
}


class Api {
    constructor() {
        this.url = new URL(document.URL)
    }

    async fetch(url) {
        return await fetch(url)
            .then((response) => { return response.json() })
            .then((response) => { return response })
            .catch(function (e) {
                console.log(e)
            })
    }

    async recent() {
        let url = new URL(this.url.protocol + this.url.host + `/api/recent/16`);
        return this.fetch(url)
    }

    async popular() {
        let url = new URL(this.url.protocol + this.url.host + `/api/popular/16`);
        return this.fetch(url)
    }

    async largest() {
        let url = new URL(this.url.protocol + this.url.host + `/api/largest/16`);
        return this.fetch(url)
    }
}

class List {
    constructor(response) {
        this.response = response
        this.sort_fn = null
        this.createListItemFn = null
    }

    toDocumentFragment() {
        let fragment = document.createDocumentFragment()
        this.response.sort(this.sort_fn)
        for (let i = 0; i < this.response.length; ++i) {
            let current = this.response[i]
            let item = this.createListItemFn(current)
            fragment.appendChild(item.toElement())
        }
        return fragment
    }
}

class RecentList extends List {
    constructor(response) {
        super(response)
        this.sort_fn = sort_recent
        this.createListItemFn = (arg) => new RecentListItem(arg)
    }
}

class PopularList extends List {
    constructor(response) {
        super(response)
        this.sort_fn = sort_popular
        this.createListItemFn = (arg) => new PopularListItem(arg)
    }
}
class LargestList extends List {
    constructor(response) {
        super(response)
        this.sort_fn = sort_largest
        this.createListItemFn = (arg) => new LargestListItem(arg)
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

async function fetchApi(apiName) {
    let Url = new URL(document.URL);
    let url = new URL(Url.protocol + Url.host + `/api/${apiName}/15`);


    fetch(url).then((response) => response.json()).then((response) => API[apiName](response))
        .catch(function (e) {
            console.log(e)
        })
}

function createExternalButton(icon) {
    let buttonGroup = document.createElement("button")

    buttonGroup.classList.add("btn", "btn-sm", "btn-outline-dark")
    buttonGroup.appendChild(icon)

    return buttonGroup
}

function createExternalButtons(repository) {
    let icon;
    if (repository.hostname === "github.com") {
        let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="18.75" height="18.75" fill="currentColor" class="bi bi-github" viewBox="-0.5 -1.5 17.5 17.5"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>`
        icon = new DOMParser().parseFromString(svg, "text/xml").firstChild

    }
    else if (repository.hostname === "gitlab.com") {
        icon = document.createElement("img")
        icon.setAttribute("width", "18px")
        icon.setAttribute("height", "22px")
        icon.setAttribute("alt", "GitLab")
        icon.setAttribute("src", "/static/gitlab-3.svg")
        icon.classList.add("float-start")
    }
    else if (repository.hostname === "bitbucket.org") {
        icon = document.createElement("img")
        icon.setAttribute("width", "20px")
        icon.setAttribute("height", "22px")
        icon.setAttribute("alt", "GitHub")
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
        icon.setAttribute("alt", "Gitea")
        icon.setAttribute("src", "/static/gitea.svg")
        icon.classList.add("float-start")
    }
    else {
        let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="18px" height="18px" style="color: #f34f29;" fill="currentColor" class="bi bi-git" viewBox="0 -1 17 17">
        <path d="M15.698 7.287 8.712.302a1.03 1.03 0 0 0-1.457 0l-1.45 1.45 1.84 1.84a1.223 1.223 0 0 1 1.55 1.56l1.773 1.774a1.224 1.224 0 0 1 1.267 2.025 1.226 1.226 0 0 1-2.002-1.334L8.58 5.963v4.353a1.226 1.226 0 1 1-1.008-.036V5.887a1.226 1.226 0 0 1-.666-1.608L5.093 2.465l-4.79 4.79a1.03 1.03 0 0 0 0 1.457l6.986 6.986a1.03 1.03 0 0 0 1.457 0l6.953-6.953a1.031 1.031 0 0 0 0-1.457"/>
        </svg>`
        icon = new DOMParser().parseFromString(svg, "text/xml").firstChild
    }
    return createExternalButton(icon)
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

    toElement() {
        let repository = this.repository
        let repository_array = repository.branches;
        let listItem = document.createElement("li");
        listItem.classList.add("list-group-item")

        let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name

        let row = createRow("align-items-center");
        let col1 = createColumn("col-sm", "text-truncate")

        let title = `${repository.repository_name}`
        let link = createExternalLink(local_href, repository.repository_name, title, "link-dark", "me-2");
        col1.appendChild(link)
        row.appendChild(col1)

        let col2 = createColumn("col-sm", "text-truncate")
        col2.appendChild(this.description)
        row.appendChild(col2)

        let href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
        let external = createExternalLink(href, "", `Open repository ${href}`, "col", "col-auto")
        let externalLink = createExternalButtons(repository);
        external.appendChild(externalLink)

        row.appendChild(external)

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

class RecentListItem extends ListItem {
    constructor(response) { super(response) }

    toElement() {
        let repository = this.repository;
        this.id = repository.repository_name + "-recent"

        let now = Date.now()
        let date = Date.parse(repository.time)
        let diff = delta_time(now, date)

        let repository_array = repository.branches

        if (repository_array.length > 1) {
            this.collapse = new RecentCollapseContent(this.id, repository).toElement()
        }
        this.description = createSmallText(diff)

        return super.toElement()
    }
}

class PopularListItem extends ListItem {
    constructor(response) { super(response) }

    toElement() {
        let repository = this.repository;
        this.id = repository.repository_name + "-popular"
        let repository_array = repository.branches;
        let totalCount = 0;
        for (let i = 0; i < repository_array.length; ++i) { totalCount += repository_array[i].count }

        if (repository_array.length > 1) {
            this.collapse = new PopularCollapseContent(this.id, repository).toElement()
        }

        this.description = createSmallText(createViewsText(totalCount))

        return super.toElement()
    }
}

class LargestListItem extends ListItem {
    constructor(response) { super(response) }

    toElement() {
        let repository = this.repository;
        this.id = this.repository.repository_name + "-largest"
        let bytes = formatBytes(repository.size)

        let repository_array = repository.branches;
        if (repository_array.length > 1) {
            this.collapse = new LargestCollapseContent(this.id, repository).toElement()
        }
        this.description = createSmallText(bytes)

        return super.toElement()
    }
}

function createExternalLink(href, innerText, title, ...classes) {
    let a = document.createElement("a")
    a.setAttribute("target", "_blank")
    a.setAttribute("rel", "noopener noreferrer canonical")
    a.setAttribute("href", href)
    a.setAttribute("title", title)
    a.innerText = innerText
    for (let i = 0; i < classes.length; i++) {
        a.classList.add(classes[i])
    }
    return a
}

function createColumn(...classes) {
    let col = document.createElement("div")
    col.classList.add("col")
    for (let i = 0; i < classes.length; i++) {
        col.classList.add(classes[i])
    }
    return col
}

function createRow(...classes) {
    let row = document.createElement("div")
    row.classList.add("row")
    if (classes.length > 0) {
        row.classList.add(classes)
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

    toElement() {
        let div = document.createElement("div");
        div.id = this.id;
        div.classList.add("collapse", "row", "mt-3");

        for (let i = 0; i < this.repository_array.length; ++i) {
            let row = createRow();

            let col1 = createColumn("col-sm", "text-truncate");

            let text = document.createElement("text");
            let branch = this.repository_array[i].branch_name;
            text.innerText = branch
            text.setAttribute("title", branch)

            col1.appendChild(text);
            row.appendChild(col1);

            let col2 = createColumn("col-auto", "text-truncate");
            col2.appendChild(this.elements[i]);
            row.appendChild(col2);

            div.appendChild(row);
        }

        return div;
    }
}
class RecentCollapseContent extends CollapseContent {
    constructor(id, repository) {
        super(id, repository);
    }

    toElement() {
        this.repository_array.sort(sort_recent)
        for (let i = 0; i < this.repository_array.length; ++i) {
            let time = this.repository_array[i].time;
            let now = Date.now()
            let date = Date.parse(time)
            let diff = delta_time(now, date)
            let small = createSmallText(diff)
            this.elements[i] = small
        }
        return super.toElement()
    }
}


class PopularCollapseContent extends CollapseContent {
    constructor(id, repository) {
        super(id, repository);
    }

    toElement() {
        this.repository_array.sort(sort_popular)
        for (let i = 0; i < this.repository_array.length; ++i) {
            let count = this.repository_array[i].count;
            let text = createViewsText(count)
            let small = createSmallText(text)
            this.elements[i] = small
        }
        return super.toElement()
    }
}

class LargestCollapseContent extends CollapseContent {
    constructor(id, repository) {
        super(id, repository);
    }

    toElement() {
        this.repository_array.sort(sort_largest)
        for (let i = 0; i < this.repository_array.length; ++i) {
            let size = this.repository_array[i].size;
            let bytes = formatBytes(size)
            let small = createSmallText(bytes)
            this.elements[i] = small
        }
        return super.toElement();
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
function createViewsText(count) { return count > 1 ? `${count} views` : `${count} view` }
function formatBytes(a, b = 2, k = 1024) { with (Math) { let d = floor(log(a) / log(k)); return 0 == a ? "0 Bytes" : parseFloat((a / pow(k, d)).toFixed(max(0, b))) + " " + ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"][d] } }

document.addEventListener("DOMContentLoaded", start);
