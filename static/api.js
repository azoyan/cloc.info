function start() {
    fetchRecent()
    fetchPopular()
    fetchLargest()
}

async function fetchRecent() {
    let Url = new URL(document.URL);
    console.log(Url)
    let url = new URL(Url.protocol + Url.host + "/api/recent/15");

    console.log(url)

    fetch(url).then((response) => response.json()).then((response) => {
        let recent = document.getElementById("recent")
        console.log(response)

        for (let i = 0; i < response.length; ++i) {
            let item = new ListItem(response[i]).recent()
            recent.appendChild(item)
        }
    }).catch(function (e) {
        console.log(e)
    })
}

async function fetchPopular() {
    let Url = new URL(document.URL);
    console.log(Url)
    let url = new URL(Url.protocol + Url.host + "/api/popular/15");

    console.log(url)

    fetch(url).then((response) => response.json()).then((response) => {
        let popular = document.getElementById("popular")

        console.log(response)

        for (let i = 0; i < response.length; ++i) {
            let item = new ListItem(response[i]).popular()
            popular.appendChild(item)
        }
    }).catch(function (e) {
        console.log(e)
    })
}

async function fetchLargest() {
    let Url = new URL(document.URL);
    console.log(Url)
    let url = new URL(Url.protocol + Url.host + "/api/largest/15");

    console.log(url)

    fetch(url).then((response) => response.json()).then((response) => {
        let largest = document.getElementById("largest")

        console.log(response)

        for (let i = 0; i < response.length; ++i) {
            let item = new ListItem(response[i]).largest()
            largest.appendChild(item)
        }
    }).catch(function (e) {
        console.log(e)
    })
}

function createExternalLink(repository, icon) {
    let href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name

    let buttonGroup = document.createElement("div")

    buttonGroup.classList.add("btn-group", "btn-group-sm", "group-hover")

    buttonGroup.setAttribute("role", "group")
    buttonGroup.setAttribute("aria-label", "Link to repository")

    let iconButton = document.createElement("button")
    iconButton.setAttribute("type", "button")
    iconButton.classList.add("btn", "btn-outline", "border", "pe-none")

    iconButton.appendChild(icon)
    buttonGroup.appendChild(iconButton)

    // let span = document.createElement("span")
    let externalButton = document.createElement("button")
    externalButton.setAttribute("type", "button")
    externalButton.classList.add("btn", "btn-outline", "border", "pe-none")

    let externalIcon = document.createElement("i")
    externalIcon.classList.add("bi", "bi-box-arrow-up-right", "align-top");
    externalButton.appendChild(externalIcon)

    buttonGroup.appendChild(externalButton)

    return buttonGroup
}

function createGitlabExternalLink(repository) {
    let gitlab_href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    let link = '<a target="_blank" rel="noopener noreferrer canonical" href="' + gitlab_href + '" class="link-dark">'
    link += `<span
    class="badge rounded-pill bg-white text-body ms-1"
    style="border: 1px solid; border-color: #CCC;">
    <img alt="Open repository" width="18px" height="18px" src="/static/gitlab-3.svg" class="float-start pe-1 ">
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
        fill="currentColor" class="bi bi-box-arrow-up-right" viewBox="0 0 16 16">
        <path fill-rule="evenodd"
            d="M8.636 3.5a.5.5 0 0 0-.5-.5H1.5A1.5 1.5 0 0 0 0 4.5v10A1.5 1.5 0 0 0 1.5 16h10a1.5 1.5 0 0 0 1.5-1.5V7.864a.5.5 0 0 0-1 0V14.5a.5.5 0 0 1-.5.5h-10a.5.5 0 0 1-.5-.5v-10a.5.5 0 0 1 .5-.5h6.636a.5.5 0 0 0 .5-.5z" />
        <path fill-rule="evenodd"
            d="M16 .5a.5.5 0 0 0-.5-.5h-5a.5.5 0 0 0 0 1h3.793L6.146 9.146a.5.5 0 1 0 .708.708L15 1.707V5.5a.5.5 0 0 0 1 0v-5z" />
    </svg>
</span>`

    link += '</a>'

    return link
}
//<img src="https://img.icons8.com/external-tal-revivo-shadow-tal-revivo/24/000000/external-bitbucket-is-a-web-based-version-control-repository-hosting-service-logo-shadow-tal-revivo.png"/>

function createBitbucketExternalLink(repository) {
    let gitlab_href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    let link = '<a target="_blank" rel="noopener noreferrer canonical" href="' + gitlab_href + '" class="link-dark">'
    link += `<span
    class="badge rounded-pill bg-white text-body ms-1"
    style="border: 1px solid; border-color: #CCC;">
    <img alt="Open repository" width="17px" height="17px    " src="/static/bitbucket-icon.svg" class="float-start pe-1 ">
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
        fill="currentColor" class="bi bi-box-arrow-up-right" viewBox="0 0 16 16">
        <path fill-rule="evenodd"
            d="M8.636 3.5a.5.5 0 0 0-.5-.5H1.5A1.5 1.5 0 0 0 0 4.5v10A1.5 1.5 0 0 0 1.5 16h10a1.5 1.5 0 0 0 1.5-1.5V7.864a.5.5 0 0 0-1 0V14.5a.5.5 0 0 1-.5.5h-10a.5.5 0 0 1-.5-.5v-10a.5.5 0 0 1 .5-.5h6.636a.5.5 0 0 0 .5-.5z" />
        <path fill-rule="evenodd"
            d="M16 .5a.5.5 0 0 0-.5-.5h-5a.5.5 0 0 0 0 1h3.793L6.146 9.146a.5.5 0 1 0 .708.708L15 1.707V5.5a.5.5 0 0 0 1 0v-5z" />
    </svg>
</span>`

    link += '</a>'
    return link
}

function createCodebergExternalLink(repository) {
    let codeberg_href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    let link = '<a target="_blank" rel="noopener noreferrer canonical" href="' + codeberg_href + '" class="link-dark">'
    // <img alt="Open repository" style="color: #2185d0" width="17px" height="17px" src="https://design.codeberg.org/logo-kit/icon_inverted.svg" class="float-start pe-1 ">

    link += `<span
    class="badge rounded-pill bg-white text-body ms-1"
    style="border: 1px solid; border-color: #CCC;">
    <svg style="color: #2185d0" role="img" width="16px" height="15px" viewBox="-3 0 27 27" xmlns="http://www.w3.org/2000/svg"><title>Codeberg</title>
    <path d="M11.955.49A12 12 0 0 0 0 12.49a12 12 0 0 0 1.832 6.373L11.838 5.928a.187.14 0 0 1 .324 0l10.006 12.935A12 12 0 0 0 24 12.49a12 12 0 0 0-12-12 12 12 0 0 0-.045 0zm.375 6.467l4.416 16.553a12 12 0 0 0 5.137-4.213z" fill="#2185d0"></path>
    </svg>
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
        fill="currentColor" class="bi bi-box-arrow-up-right" viewBox="0 0 16 16">
        <path fill-rule="evenodd"
            d="M8.636 3.5a.5.5 0 0 0-.5-.5H1.5A1.5 1.5 0 0 0 0 4.5v10A1.5 1.5 0 0 0 1.5 16h10a1.5 1.5 0 0 0 1.5-1.5V7.864a.5.5 0 0 0-1 0V14.5a.5.5 0 0 1-.5.5h-10a.5.5 0 0 1-.5-.5v-10a.5.5 0 0 1 .5-.5h6.636a.5.5 0 0 0 .5-.5z" />
        <path fill-rule="evenodd"
            d="M16 .5a.5.5 0 0 0-.5-.5h-5a.5.5 0 0 0 0 1h3.793L6.146 9.146a.5.5 0 1 0 .708.708L15 1.707V5.5a.5.5 0 0 0 1 0v-5z" />
    </svg>
</span>`

    link += '</a>'
    return link
}

function createGitExternalLink(repository) {
    let gitlab_href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    let link = '<a target="_blank" rel="noopener noreferrer canonical" href="' + gitlab_href + '" class="link-dark">'
    link += `<span
    class="badge rounded-pill bg-white text-body ms-1"
    style="border: 1px solid; border-color: #CCC;">
    <svg xmlns="http://www.w3.org/2000/svg" width="16" style="color: #f34f29;" height="16" fill="currentColor" class="bi bi-git" viewBox="0 0 16 16">
  <path d="M15.698 7.287 8.712.302a1.03 1.03 0 0 0-1.457 0l-1.45 1.45 1.84 1.84a1.223 1.223 0 0 1 1.55 1.56l1.773 1.774a1.224 1.224 0 0 1 1.267 2.025 1.226 1.226 0 0 1-2.002-1.334L8.58 5.963v4.353a1.226 1.226 0 1 1-1.008-.036V5.887a1.226 1.226 0 0 1-.666-1.608L5.093 2.465l-4.79 4.79a1.03 1.03 0 0 0 0 1.457l6.986 6.986a1.03 1.03 0 0 0 1.457 0l6.953-6.953a1.031 1.031 0 0 0 0-1.457"/>
</svg>
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
        fill="currentColor" class="bi bi-box-arrow-up-right" viewBox="0 0 16 16">
        <path fill-rule="evenodd"
            d="M8.636 3.5a.5.5 0 0 0-.5-.5H1.5A1.5 1.5 0 0 0 0 4.5v10A1.5 1.5 0 0 0 1.5 16h10a1.5 1.5 0 0 0 1.5-1.5V7.864a.5.5 0 0 0-1 0V14.5a.5.5 0 0 1-.5.5h-10a.5.5 0 0 1-.5-.5v-10a.5.5 0 0 1 .5-.5h6.636a.5.5 0 0 0 .5-.5z" />
        <path fill-rule="evenodd"
            d="M16 .5a.5.5 0 0 0-.5-.5h-5a.5.5 0 0 0 0 1h3.793L6.146 9.146a.5.5 0 1 0 .708.708L15 1.707V5.5a.5.5 0 0 0 1 0v-5z" />
    </svg>
</span>`

    link += '</a>'
    return link
}

function createExternalLinks(repository) {
    console.log("createExternalLinks", repository)

    let icon;
    if (repository.hostname === "github.com") {
        let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-github" viewBox="0 0 16 16"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>`
        icon = new DOMParser().parseFromString(svg, "text/xml").firstChild
    }
    else if (repository.hostname === "gitlab.com") {
        icon = document.createElement("img")
        icon.setAttribute("alt", "Open repository")
        icon.setAttribute("width", "18px")
        icon.setAttribute("height", "18px")
        icon.setAttribute("src", "/static/gitlab-3.svg")
        icon.classList.add("float-start", "pe-1")
    }
    else if (repository.hostname === "bitbucket.org") {
        icon = document.createElement("img")
        icon.setAttribute("alt", "Open repository")
        icon.setAttribute("width", "18px")
        icon.setAttribute("height", "18px")
        icon.setAttribute("src", "/static/bitbucket-icon.svg")
        icon.classList.add("float-start", "pe-1")
    }
    else if (repository.hostname == "codeberg.org") {
        let svg = `<svg style="color: #2185d0" role="img" width="16px" height="15px" viewBox="-3 0 27 27" xmlns="http://www.w3.org/2000/svg"><title>Codeberg</title>
                <path d="M11.955.49A12 12 0 0 0 0 12.49a12 12 0 0 0 1.832 6.373L11.838 5.928a.187.14 0 0 1 .324 0l10.006 12.935A12 12 0 0 0 24 12.49a12 12 0 0 0-12-12 12 12 0 0 0-.045 0zm.375 6.467l4.416 16.553a12 12 0 0 0 5.137-4.213z" fill="#2185d0"></path>
            </svg>`
        icon = new DOMParser().parseFromString(svg, "text/xml").firstChild
    }
    else {
        return createGitExternalLink(repository)
    }
    return createExternalLink(repository, icon)
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

        this.description = createSmallText(`${totalCount} views`)
        return this.toElement()
    }

    recent() {
        let repository = this.repository;
        this.id = repository.repository_name + "-recent"

        let now = Date.now()
        let date = Date.parse(repository.time)
        let diff = delta_time(now, date)
        
        let repository_array = repository.branches;
        console.log("Recent", repository_array)
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

        col1.appendChild(this.description)

        let col3 = document.createElement("a")
        col3.setAttribute("target", "_blank")
        col3.setAttribute("rel", "noopener noreferrer canonical")
        let href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
        col3.setAttribute("href", href)
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
        } else {
            listItem.appendChild(row)
        }

        return listItem
    }
}

function createLargestItem(repository) {
    console.log("createLargest", repository)
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
    console.log("total count", totalCount)

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
    if (tokens.length > 0) {
        col.classList.add(tokens)
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
        for (let i = 0; i < this.repository_array.length; ++i) {
            let size = this.repository_array[i].size;
            let bytes = formatBytes(size)
            let small = createSmallText(bytes)
            this.elements[i] = small
        }
        return this.toElement();
    }

    popular() {
        for (let i = 0; i < this.repository_array.length; ++i) {
            let count = this.repository_array[i].count;
            let text = `${count} view`
            let small = createSmallText(text)
            this.elements[i] = small
        }
        return this.toElement()
    }

    toElement() {
        console.log("toElement")
        let div = document.createElement("div");
        div.id = this.id;
        div.classList.add("collapse", "row");

        for (let i = 0; i < this.repository_array.length; ++i) {
            let row = createRow();

            let col1 = createColumn("col-auto");
            row.appendChild(col1);

            let a = document.createElement("text");
            a.innerText = this.repository_array[i].branch_name;

            col1.appendChild(a);
            let col2 = createColumn("col-auto");
            col2.appendChild(this.elements[i]);
            row.appendChild(col2);

            div.appendChild(row);
        }

        return div;
    }
}

function createCollapseContent(id, repository_array, s, key) {
    let div = document.createElement("div")
    div.id = id
    div.classList.add("collapse", "row")

    for (let i = 0; i < repository_array.length; ++i) {
        let row = createRow()
        let r = repository_array[i]

        let col1 = createColumn("col-auto")
        row.appendChild(col1)

        let a = document.createElement("text")

        a.innerText = repository_array[i].branch_name


        col1.appendChild(a)
        let smallText = createSmallText(s.replace("{ }", r[key]))

        let col2 = createColumn("col-auto")
        col2.appendChild(smallText)
        row.appendChild(col2)
        // row.appendChild(createExternalLinks(repository_array[i]))

        div.appendChild(row)
    }

    return div
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
    icon.classList.add("bi", "bi-chevron-down")

    button.appendChild(icon)
    return button
}

function delta_time(now, date) {
    let dt = (now - date) / 1000
    console.log(dt)
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

function formatBytes(a, b = 2, k = 1024) { with (Math) { let d = floor(log(a) / log(k)); return 0 == a ? "0 Bytes" : parseFloat((a / pow(k, d)).toFixed(max(0, b))) + " " + ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"][d] } }

document.addEventListener("DOMContentLoaded", start);
