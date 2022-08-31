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
    <img alt="Open repository" src="/static/GitHub16.png" class="float-start pe-1 ">
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

function createGitlabExternalLink(repository) {
    let gitlab_href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    let link = '<a target="_blank" rel="noopener noreferrer canonical" href="' + gitlab_href + '" class="link-dark">'
    link += `<span
    class="badge rounded-pill bg-white text-body ms-1"
    style="border: 1px solid; border-color: #CCC;">
    <img alt="Open repository" src="/static/gitlab-16c.png" class="float-start pe-1 ">
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
    <img alt="Open repository" src="/static/bitbucket16.png" class="float-start pe-1 ">
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
    <img alt="Open repository" src="/static/git16.png" class="float-start pe-1 ">
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
    if (repository.repository_name.slice(-4) === ".git") {
        repository.repository_name = repository.repository_name.slice(0, -4);
    }
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
    else if (repository.hostname === "gitlab.com") {
        item += createGitlabExternalLink(repository)
    }
    else if (repository.hostname === "bitbucket.org") {
        item += createBitbucketExternalLink(repository)
    }
    else {
        item += createGitExternalLink(repository)
    }

    item += '</li>'
    return item
}

function createPopularListItem(repository) {
    let item = '<li class="list-group-item d-flex align-items-center">'
    if (repository.repository_name.slice(-4) === ".git") {
        repository.repository_name = repository.repository_name.slice(0, -4)
    }
    let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    item += '<a target="_blank" rel="noopener noreferrer canonical" href="' + local_href + '" class="link-dark me-1">' + repository.repository_name + '</a>'


    let count = "(" + repository.count + " views)"


    item += count

    if (repository.hostname === "github.com") {
        item += createGithubExternalLink(repository)
    }
    else if (repository.hostname === "gitlab.com") {
        item += createGitlabExternalLink(repository)
    }
    else if (repository.hostname === "bitbucket.org") {
        item += createBitbucketExternalLink(repository)
    }
    else {
        item += createGitExternalLink(repository)
    }

    item += '</li>'
    return item
}

function createLargestListItem(repository) {
    if (repository.repository_name.slice(-4) === ".git") {
        repository.repository_name = repository.repository_name.slice(0, -4);
    }
    let item = '<li class="list-group-item d-flex align-items-center">'
    let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name
    item += '<a target="_blank" rel="noopener noreferrer canonical" href="' + local_href + '" class="link-dark me-1">' + repository.repository_name + '</a>'


    let size = "(" + formatBytes(repository.size) + ")"


    item += size

    if (repository.hostname === "github.com") {
        item += createGithubExternalLink(repository)
    }
    else if (repository.hostname === "gitlab.com") {
        item += createGitlabExternalLink(repository)
    }
    else if (repository.hostname === "bitbucket.org") {
        item += createBitbucketExternalLink(repository)
    }
    else {
        item += createGitExternalLink(repository)
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