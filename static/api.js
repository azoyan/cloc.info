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
        let items = ""
        console.log(response)
        for (let i = 0; i < response.length; ++i) {
            let item = createRecentListItem(response[i])
            items += item
        }
        recent.innerHTML = items
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
        let recent = document.getElementById("popular")
        let items = ""
        console.log(response)
        for (let i = 0; i < response.length; ++i) {
            let item = createPopularListItem(response[i])
            items += item
        }
        recent.innerHTML = items
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
        let recent = document.getElementById("largest")
        let items = ""
        console.log(response)
        for (let i = 0; i < response.length; ++i) {
            let item = createLargestListItem(response[i])
            items += item
        }
        recent.innerHTML = items
    }).catch(function (e) {
        console.log(e)
    })
}

function createGithubExternalLink(repository) {
    let github_href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    let link = '<a target="_blank" rel="noopener noreferrer canonical" href="' + github_href + '" class="link-dark">'
    link += `<span
    class="badge rounded-pill bg-white text-body ms-1"
    style="border: 1px solid; border-color: #CCC;">
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-github" viewBox="0 0 16 16">
  <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
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

function createRecentListItem(repository) {
    let item = '<li class="list-group-item d-flex align-items-center">'
    let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    item += '<a target="_blank" rel="noopener noreferrer canonical" href="' + local_href + '" class="link-dark me-1">' + repository.repository_name + '</a>'

    let now = Date.now()
    let date = Date.parse(repository.time)
    let diff = delta_time(now, date)

    item += diff

    if (repository.hostname === "github.com") {
        item += createGithubExternalLink(repository)
    }

    item += '</li>'
    return item
}

function createPopularListItem(repository) {
    let item = '<li class="list-group-item d-flex align-items-center">'
    let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    item += '<a target="_blank" rel="noopener noreferrer canonical" href="' + local_href + '" class="link-dark me-1">' + repository.repository_name + '</a>'


    let count = "(" + repository.count + " views)"


    item += count

    if (repository.hostname === "github.com") {
        item += createGithubExternalLink(repository)
    }

    item += '</li>'
    return item
}

function createLargestListItem(repository) {
    let item = '<li class="list-group-item d-flex align-items-center">'
    let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    item += '<a target="_blank" rel="noopener noreferrer canonical" href="' + local_href + '" class="link-dark me-1">' + repository.repository_name + '</a>'


    let size = "(" + formatBytes(repository.size) + ")"


    item += size

    if (repository.hostname === "github.com") {
        item += createGithubExternalLink(repository)
    }

    item += '</li>'
    return item
}


function delta_time(now, date) {
    let dt = (now - date) / 1000
    console.log(dt)
    if (dt > 60 && dt < 7200) {
        dt = "(" + Math.round(dt / 60) + " minutes ago)"
    }
    else if (dt > 7200 && dt < 86400) {
        dt = "(" + Math.round(dt / 3600) + " hours ago)"
    }
    else if (dt > 86400) {
        dt = "(" + Math.round(dt / 86400) + " days ago)"
    }
    else {
        dt = "(" + Math.round(dt) + " seconds ago)"
    }
    return dt
}

function formatBytes(a, b = 2, k = 1024) { with (Math) { let d = floor(log(a) / log(k)); return 0 == a ? "0 Bytes" : parseFloat((a / pow(k, d)).toFixed(max(0, b))) + " " + ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"][d] } }

document.addEventListener("DOMContentLoaded", start);