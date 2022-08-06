const input = document.getElementById('input');

let Later = null;
let hint_url = document.getElementById("hint_url");

const submitButton = document.getElementById('submitButton');

submitButton.onclick = function () {
    let url = new URL(input.value)
    let selected = document.getElementById("select-child").value;
    console.log(selected)
    let path;
    if (selected === branches.default_branch) {
        path = url.host + url.pathname + url.search + url.hash

    }
    else {
        if (url.host === "github.com") {
            path = url.host + url.pathname + url.search + url.hash + '/tree/' + selected
        }
    }
    window.location = path
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
    let url;
    hint.style.display = 'invisibles'

    if (!url_str.match(/^[a-zA-Z]+:\/\//)) { url_str = 'https://' + url_str; }
    console.log(url_str);
    
    try {
        url = new URL(url_str);
    }
    catch (error) {
        console.log("Invalid URL:", error);
        document.getElementById("invalidFeedback").innerText = '"' + url_str + '" is not valid URL.'
        document.getElementById("input").classList.add("is-invalid");
        return
    }
    
    document.getElementById("buttonText").innerText = "Check"
    let checkSpinner = document.getElementById("checkSpinner");
    checkSpinner.hidden = false;

    let submitButton = document.getElementById("submitButton");
    

    let api_url = document.URL + "api/" + url.hostname + url.pathname + "/branches";
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
            document.getElementById("commit").innerHTML = '<p class="font-monospace text-truncate">' + branches[i].commit.sha
        }
        else {
            select += createSelectOption(branchName)
        }
    }
    if (input.value.includes("https://github.com")) {
        document.getElementById("github_picture").classList.add("visible")
        document.getElementById("github_picture").classList.remove("invisible")
    }
    else if (input.value.includes("https://gitlab.com")) {
        document.getElementById("gitlab_picture").hidden = false
    }
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