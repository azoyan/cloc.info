const input = document.getElementById('input');

let Later = null;
let hint_url = document.getElementById("hint_url");

const submitButton = document.getElementById('submitButton');

submitButton.onclick = function () {
    let url = gitUrlParse(input.value)
    let selected = document.getElementById("select-child").value;
    console.log(selected)
    console.log(url)
    let path =
        url.host + '/' + url.owner + '/' + url.name + '.git'
    if (selected !== branches.default_branch) {
        if (url.host === "github.com" || url.host==="gitlab.com") {
            path += '/tree/' + selected
        }
    }

    path = path.replace(/\/\//g, "/")

    console.log("path", path);

    window.location.href = path
}

hint_url.onclick = function () {
    input.value = hint_url.innerText
    check(input.value);
}

input.oninput = (evt) => {
    const isPasted = evt.inputType && evt.inputType.startsWith("insertFromPaste");
    if (isPasted) {
        pasteValue(evt)
    } else {
        editValue(evt)
    }
};

function reset() {
    console.log("reset");
    document.getElementById("input").classList.remove("is-valid");
    document.getElementById("input").classList.remove("is-invalid");
    hint.style.removeProperty('display');
    document.getElementById("select").innerHTML = ''
    document.getElementById("repoInfo").classList.add("invisible")
    document.getElementById("repository_pic").classList.remove("visible");
    document.getElementById("repository_pic").classList.add("invisible")

    document.getElementById("hint").classList.remove("invisible")
    document.getElementById("hint").classList.add("visible")

    submitButton.classList.add("disabled");
    submitButton.classList.add('btn-outline-success');
    submitButton.classList.remove("btn-success")
}

async function editValue(e) {
    if (Later != null) {
        Later.cancel();
        Later = null;
    }
    if (e.target.value === "") {
        return reset();
    }

    if (Later === null) {
        Later = later(2000, false)
        Later.promise
            .then(function () {
                console.log(e.target.value)
                check(e.target.value)
            })
            .catch((e) => { console.log("later cancelled", e); });
    }
}

function pasteValue(e) {
    if (e.target.value === "") return;
    check(e.target.value)
}

let branches;

function check(url_str) {
    hint.style.display = 'invisible'

    let git_extension = url_str.slice(-4);
    if (git_extension !== ".git") {
        url_str += ".git"
    }

    let is_git_regex = /(?:git|ssh|https?|git@[-\w.]+):(\/\/)?(.*?)(\.git)(\/?|\#[-\d\w._]+?)$/;
    // if (!url_str.match(is_git_regex)) { url_str = 'https://' + url_str; }
    // console.log(url_str);
    let parsed_url = gitUrlParse(url_str)
    console.log("parsed:", parsed_url)
    if (url_str.match(is_git_regex)) {
        console.log("valid git url", url_str)
    }
    else {
        console.log("Invalid URL:", error);
        document.getElementById("invalidFeedback").innerText = '"' + url_str + '" is not valid URL.'
        document.getElementById("input").classList.add("is-invalid");
        return
    }

    document.getElementById("buttonText").innerText = "Check"
    let checkSpinner = document.getElementById("checkSpinner");
    checkSpinner.hidden = false;

    let submitButton = document.getElementById("submitButton");

    let api_url = document.URL + "api/" + parsed_url.host + parsed_url.pathname + "/branches";
    api_url = api_url.replace(/([^:]\/)\/+/g, "$1");

    console.log("api_url", api_url)

    fetch(api_url)
        .then((response) => response.json())
        .then((response) => {
            branches = response;

            let select = document.getElementById("select");
            let html_select = createSelect(branches, "select-child");
            select.appendChild(createElementFromHTML(html_select));

            submitButton.classList.remove("disabled");
            submitButton.classList.remove('btn-outline-success');
            submitButton.classList.add("btn-success");

            document.addEventListener("keypress", function (event) {
                if (event.key === "Enter") {
                    event.preventDefault();
                    submitButton.click();
                }
            });

            document.getElementById("input").classList.add("is-valid");
            document.getElementById("input").classList.remove("is-invalid");
            document.getElementById("invalidFeedback").classList.remove("is-invalid");
            document.getElementById("invalidFeedback").classList.add("invisible");
            document.getElementById("repoInfo").classList.remove("invisible");
            document.getElementById("buttonText").innerText = "Submit"
            document.getElementById("hint").classList.add("invisible")
            checkSpinner.hidden = true;
        })
        .catch(function (error) {
            document.getElementById("invalidFeedback").innerText = error
            checkSpinner.hidden = true;
            submitButton.classList.add("disabled");
            submitButton.classList.add('btn-outline-success');
            document.getElementById("buttonText").innerText = "Submit"
            document.getElementById("repoInfo").classList.add("invisible");
            document.getElementById("invalidFeedback").classList.remove("invisible");
            document.getElementById("input").classList.add("is-invalid");
            if (error.response) {
                console.error(error.response.data);
                console.error(error.response.status);
                console.error(error.response.headers);
            }
            else {
                console.log("error", error)
            }
        });
}

function createSelect(all_branches, id) {
    let branches = all_branches.branches;
    let defaultBranch = all_branches.default_branch;
    document.getElementById(id)?.remove() // delete previous if exists
    let select = '<select class="form-select form-select-sm" aria-label=".form-select-sm example" id="' + id + '" onchange="setCommit(this.value)">'
    for (var i = 0; i < branches.length; ++i) {
        let branchName = branches[i].name
        if (branchName === defaultBranch) {
            select += createSelectOption(branchName, true)
            document.getElementById("commit").innerHTML = '<p class="font-monospace text-truncate" data-bs-toggle="tooltip" data-bs-placement="top" data-bs-title="Last commit">' + branches[i].commit
        }
        else {
            select += createSelectOption(branchName)
        }
    }
    let pic = document.getElementById("repository_pic")
    pic.classList.add("visible")
    pic.classList.remove("invisible")
    pic.setAttribute("data-bs-toggle", "tooltip")
    pic.setAttribute("data-bs-title", "Go to repository " + input.value)
    pic.setAttribute("data-bs-placement", "top")
    pic.setAttribute("href", input.value)
    if (input.value.includes("github.com")) { pic.innerHTML = '<img src="static/GitHub-Mark-32px.png" class="float-start">' }
    else if (input.value.includes("gitlab.com")) { pic.innerHTML = '<img src="static/gitlab32.png" class="float-start">' }
    else if (input.value.includes("bitbucket.org")) { pic.innerHTML = '<img src="static/bitbucket.png" class="float-start">' }
    else { pic.innerHTML = '<img src="static/git32.png" class="float-start">' }

    const tooltipTriggerList = document.querySelectorAll('[data-bs-toggle="tooltip"]')
    const tooltipList = [...tooltipTriggerList].map(tooltipTriggerEl => new bootstrap.Tooltip(tooltipTriggerEl))
    select += '</select>'
    return select;
}

function substitute() {
    pasteValue("https://github.com/actix/actix-web")
}

function setCommit(branchName) {
    // let branchName = e.value
    for (let i = 0; i < branches.length; ++i) {
        if (branches[i].name === branchName) {
            document.getElementById("commit").innerText = "Last commit: " + branches[i].commit.sha
        }
    }
}

function createSelectOption(branch, isMain) {
    if (isMain) {
        return "<option selected>" + branch + "</option>"
    }
    else {
        return "<option>" + branch + "</option>"
    }
}
function createElementFromHTML(htmlString) {
    var div = document.createElement('div');
    div.innerHTML = htmlString.trim();

    // Change this to div.childNodes to support multiple top-level nodes.
    return div.firstChild;
}