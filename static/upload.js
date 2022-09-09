document.getElementById("file-input").onchange = e => {
    handleFiles(e.target.files)
}

function dropHandler(ev) {
    let files = []
    console.log('File(s) dropped');
    ev.preventDefault();
    if (ev.dataTransfer.items) {
        // Use DataTransferItemList interface to access the file(s)
        [...ev.dataTransfer.items].forEach((item, i) => {
            // If dropped items aren't files, reject them
            if (item.kind === 'file') {
                const file = item.getAsFile();
                files[i] = file
                console.log(`… file[${i}].name = ${file.name}`);
            }
        });
    } else {
        // Use DataTransfer interface to access the file(s)
        [...ev.dataTransfer.files].forEach((file, i) => {
            console.log(`… file[${i}].name = ${file.name}`);
            files[i] = file
        });
    }

    handleFiles(files)
}

function handleFiles(files) {
    if (files.length > 0) {
        const submitButton = document.getElementById("submit")
        submitButton.hidden = false
        let itemsWidget = document.getElementById("items");
        itemsWidget.innerHTML += '<h5>Files to upload (' + files.length + ' items):</h5>'
        itemsWidget.innerHTML += '<ul class="list-group list-group-numbered">'
        for (let i = 0; i < files.length; ++i) {
            itemsWidget.innerHTML += '<li class="list-group-item">' + files[i].name + '</li>'
        }
        itemsWidget.innerHTML += '</ul>'

        const area = document.getElementById("area")
        area.style.border = "1px silver solid"
        area.style.cursor = "unset"
        area.onclick = () => false
        document.getElementById("label").hidden = true
        document.getElementById("label2").hidden = true

        submitButton.addEventListener('click', function (event) {
            document.getElementById("processing").hidden = false
            const progress = document.getElementById('progress-bar');
            const data = new FormData();
            let delta = Math.floor(100 / files.length)
            for (const file of files) {
                data.append(file.name, file, file.name);
            }

            const request = new XMLHttpRequest();
            request.open('POST', '/post');
            let i = 0
            document.getElementById("status").innerHTML += '<div class="card-text p-0">Load ' + files[i].name + '</div>'
            request.upload.addEventListener('progress', function (e) {
                const percent = Math.floor((e.loaded / e.total) * 100)
                if (files[i] !== undefined && Math.floor(percent / delta) === 0) {
                    i += 1
                    document.getElementById("status").innerHTML += '<div class="card-text p-0">Load ' + files[i].name + '</div>'
                }
                progress.style.width = percent + '%';
            })

            request.addEventListener('load', function () {
                console.log(request.status);
                createTableFromResponse(request.response)
                document.getElementById("area").hidden = true
                document.getElementById("processing").hidden = true;
            })

            submitButton.hidden = true
            request.send(data);
        })
    }
}

function dragOverHandler(ev) {
    let files = []
    if (ev.dataTransfer.items) {
        [...ev.dataTransfer.items].forEach((item, i) => {
            // If dropped items aren't files, reject them
            console.log(item)
            if (item.kind === 'file') {
                const file = item.getAsFile();
                if (file !== null) {
                    files.push(file)
                    console.log(`… file[${i}].name = ${file.name}`);
                }
            } else if (item.name) {
                files.push({ name: item.name })
            }
        });
    } else {
        [...ev.dataTransfer.files].forEach((file, i) => {
            console.log(`… file[${i}].name = ${file.name}`);
            files.push(file)
        });
    }

    const label = document.getElementById("label")
    const label2 = document.getElementById("label2")
    label2.innerText = ""
    // label.innerHTML = "<h5> " + files.length + " files </h5>"
    // label.innerHTML += '<ul class="list-group list-group-numbered">'
    // for (let i = 0; i < files.length; ++i) {
    //     label.innerHTML += '<li class="list-group-item">' + files[i].name + '</li>'
    // }
    // label.innerHTML += '</ul>'
    label.innerText = "Dropping " + ev.dataTransfer.items.length + " files"
    ev.preventDefault();
}
