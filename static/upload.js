function request(e) {
    let url = new URL(document.URL);

    fetch(url,
        {
            headers: {
                'If-Match': 'cloc'
            }
        })
        .then(response => {
            return response.text()
        })
        .then(startStreaming)
        .catch(function (e) {
            console.log(e);
        });
}

document.onload = request

function startStreaming(data) {
    let ws = new WebSocket(data);

    ws.onmessage = function (event) {
        let payload = event.data
        console.log(payload)
    }
}

function createTableFromResponse(data) {
    console.log(data)
    let strings = data.split("\n")
    strings.splice(0, 1);
    strings.splice(1, 1);
    strings.splice(strings.length - 3, 3)
    strings.splice(strings.length - 4, 1)
    strings.splice(strings.length - 5, 1)
    let cocoma = strings.splice(strings.length - 3, 3);

    for (let i = 0; i < strings.length; ++i) {
        let array = strings[i].split(/\s+/);
        while (array.length > 8) {
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
    table += "</table>"
    document.getElementById("t").innerHTML = table
    console.log(strings, cocoma)

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

document.addEventListener("DOMContentLoaded", request);