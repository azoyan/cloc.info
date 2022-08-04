function start() {
    console.log("her")
    fetchRecent();
}

async function fetchRecent() {
    let Url = new URL(document.URL);
    console.log(Url)
    let url = new URL(Url.protocol + Url.host + "/api/recent/15");

    console.log(url)

    fetch(url).then((response) => response.json()).then((response) => {
        let recent = document.getElementById("recent")
        let rows = ""
        console.log(response)
        for (let i = 0; i < response.length; ++i) {
            let row = '<div class="row">'
            let now = Date.now()
            let date = Date.parse(response[i].time)
            let diff = delta_time(now, date)
            row += response[i].repository_name + " " + diff
            console.log("Date = ", date, now)

            // rows += diff
            row += "</div>"
            rows += row
        }
        recent.innerHTML = rows
    }).catch(function (e) {
        console.log(e)
    })
}

function delta_time(now, date) {
    let dt = (now - date) / 1000
    console.log(dt)
    if (dt > 60 && dt < 10800) {
        dt = "(" + Math.floor(dt / 60) + " minutes ago)"
    }
    else if (dt > 10800 && dt < 86400) {
        dt = "(" + Math.floor(dt / 3600) + " hours ago)"
    }
    else if (dt > 86400) {
        dt ="(" + Math.floor(dt / 86400) + " days ago)"
    }
    else {
        dt = "(" + dt + " seconds ago)"
    }
    return dt

}

document.addEventListener("DOMContentLoaded", start);