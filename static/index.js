const input = document.getElementById('input');

let Later = null;
let hint_url = document.getElementById("hint_url");

const submitButton = document.getElementById('submitButton');

submitButton.onclick = function () {
    let url = new URL(input.value)
    let path = url.host + url.pathname + url.search + url.hash
    console.log(path)
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

function check(url_str) {
    let url;
    hint.style.display = 'none'
    
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

    let checkSpinner = document.getElementById("checkSpinner");
    checkSpinner.classList.remove("invisible");

    let submitButton = document.getElementById("submitButton");

    let api_url = document.URL + "api/" + url.hostname + url.pathname + "/branches";
    console.log("api_url", api_url)

    fetch(api_url)
        .then((response) => { return response.json() })
        .then((response) => {
            let branches = response;

            branches.map((branch) => console.log("name = ", branch.name))
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
            checkSpinner.classList.add("invisible");
        })
        .catch(function (error) {
            document.getElementById("invalidFeedback").innerText = error
            checkSpinner.classList.add("invisible");
            if (error.response) {
                console.log(error.response.data);
                console.log(error.response.status);
                console.log(error.response.headers);
                document.getElementById("input").classList.add("is-invalid");
            }
            else { console.log("error", error) }
        });
}

function createSelect(branches, id) {
    document.getElementById(id)?.remove()
    let select = '<select class="form-select form-select-sm" aria-label=".form-select-sm example" id="' + id + '">'
    let hasMain = false;
    for (var i = 0; i < branches.length; ++i) {
        if (branches[i].name === "main" && hasMain === false) {
            select += createSelectOption(branches[i].name, true)
            hasMain = true;
        }
        else if (branches[i].name === "master") {
            if (!hasMain) {
                select += createSelectOption(branches[i].name, true)
                hasMain = true;
            }
        }
        else {
            select += createSelectOption(branches[i].name)
        }
    }
    select += '</select>'
    return select;
}

function substitute() {
    pasteValue("https://github.com/actix/actix-web")
}

function createSelectOption(branch, main) {
    if (main) {
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