function createTableFromResponse(data) {
    let strings = data.split("\n")
    console.log(data)

    strings.splice(0, 1);
    strings.splice(1, 1);
    console.log(strings.splice(-1, 1))

    console.log(strings.splice(-2, 2))

    let processed = strings.splice(-1, 1)
    // console.log(processed)
    console.log(strings.splice(-1, 1))
    let cocomo = strings.splice(-3, 3);
    // console.log(cocomo)
    console.log(strings.splice(-1))
    console.log(strings.splice(-2, 1))

    for (let i = 0; i < strings.length; ++i) {
        let array = strings[i].trim().split(/\s+/);
        while (array.length > 7) {
            array[0] += array[1]
            array.splice(1, 1)
        }
        strings[i] = array;
    }

    let table = '<table class="table">'
    table += createTableThead(strings[0])
    table += "<tbody>"

    for (let i = 1; i < strings.length; ++i) {
        table += createTableRow(strings[i])
    }


    table += "</tbody>"
    let caption = '<caption>' + processed + '</caption>'
    table += caption
    table += "</table>"
    document.getElementById("t").innerHTML = table
    document.getElementById("t").hidden = false
    console.log(strings, cocomo)
    createCocomoFromResponse(cocomo)
}

function createTableThead(array) {
    let thead = '<thead style="border-color:black;"><tr>'
    for (let i = 0; i < array.length; ++i) {
        thead += '<th scope="col">' + array[i] + '</th>'
    }
    thead += "</tr></thead>"
    return thead;
}

function createTableRow(array) {
    let row = "<tr>"

    row += '<th scope="row">' + array[0] + '</th>'

    for (let i = 1; i < array.length; ++i) {
        row += "<td>" + array[i] + "</td>"
    }
    row += "</tr>"
    return row
}

function createCocomoFromResponse(cocomo_data) {
    let str = ""

    str += '<div class="card-body"><h5 class="card-title"><strong>COCOMO</strong></h5><h6 class="card-subtitle mb-4 text-muted">Constructive Cost Model (<a target="_blank" rel="noopener noreferrer canonical" href="https://en.wikipedia.org/wiki/COCOMO">wiki</a>)</h6>'
    str += '<p class="card-text">' + cocomo_data[0] + '</p>'
    str += '<p class="card-text">' + cocomo_data[1] + '</p>'
    str += '<p class="card-text">' + cocomo_data[2] + '</p>'
    str += '</div>'

    let cocomo = document.getElementById("cocomo");
    cocomo.innerHTML = str
    cocomo.hidden = false
}

function createRepositoryIcon(input, width, height) {
    if (input.includes("github.com")) { return `<img width=${width} height=${height} src="/static/github-icon-1.svg" alt="Open repository" class="float-start">` }
    else if (input.includes("gitlab.com")) { return `<img width=${width} height=${height} src="/static/gitlab-3.svg" alt="Open repository" class="float-start">` }
    else if (input.includes("bitbucket.org")) { return `<img width=${width} height=${height} src="/static/bitbucket-icon.svg" alt="Open repository" class="float-start">` }
    else if (input.includes("codeberg.org")) { return `<img width=${width} height=${height} src="/static/codeberg-svgrepo-com.svg" alt="Open repository" class="float-start">` }
    else if (input.includes("gitea.com")) { return `<img width=${width} height=${height} src="/static/gitea.svg" alt="Open repository" class="float-start">` }
    else { return `<img width=${width} height=${height} src="/static/git-icon.svg" alt="Open repository" class="float-start">` }
}
