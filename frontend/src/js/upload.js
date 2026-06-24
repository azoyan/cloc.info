import { createTableFromResponse } from "./common.js";

const fileInput = document.getElementById("file-input")
const directoryInput = document.getElementById("directory-input")
const dropArea = document.getElementById("area")
const filePickerButton = document.getElementById("file-picker-button")
const directoryPickerButton = document.getElementById("directory-picker-button")

function openFileBrowser() {
    if (dropArea.dataset.hasFiles === "true") {
        return
    }

    fileInput.click()
}

fileInput.onchange = e => {
    handleFiles(normalizeUploadEntries(e.target.files))
    e.target.value = ""
}

directoryInput.onchange = e => {
    handleFiles(normalizeUploadEntries(e.target.files, true))
    e.target.value = ""
}

dropArea.addEventListener('drop', dropHandler)
dropArea.addEventListener('dragover', dragOverHandler)
dropArea.addEventListener('click', openFileBrowser)
filePickerButton.addEventListener('click', (ev) => {
    ev.preventDefault()
    ev.stopPropagation()
    fileInput.click()
})
directoryPickerButton.addEventListener('click', async (ev) => {
    ev.preventDefault()
    ev.stopPropagation()
    await browseDirectory()
})

function createUploadEntry(file, uploadName = file.name) {
    return {
        file,
        uploadName,
        displayName: uploadName,
    }
}

function normalizeUploadEntries(files, preserveRelativePath = false) {
    return [...files]
        .filter(Boolean)
        .map((file) => createUploadEntry(
            file,
            preserveRelativePath && file.webkitRelativePath ? file.webkitRelativePath : file.name,
        ))
}

async function browseDirectory() {
    if (typeof window.showDirectoryPicker === 'function') {
        try {
            const directoryHandle = await window.showDirectoryPicker()
            const entries = []
            await collectDirectoryEntries(directoryHandle, directoryHandle.name, entries)
            handleFiles(entries)
        } catch (error) {
            if (error && error.name !== 'AbortError') {
                console.error(error)
            }
        }

        return
    }

    directoryInput.click()
}

async function collectDirectoryEntries(directoryHandle, prefix, entries) {
    for await (const handle of directoryHandle.values()) {
        if (handle.kind === 'file') {
            const file = await handle.getFile()
            const uploadName = prefix ? `${prefix}/${file.name}` : file.name
            entries.push(createUploadEntry(file, uploadName))
            continue
        }

        const nextPrefix = prefix ? `${prefix}/${handle.name}` : handle.name
        await collectDirectoryEntries(handle, nextPrefix, entries)
    }
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
                if (file !== null) {
                    files[i] = createUploadEntry(file)
                    console.log(`… file[${i}].name = ${file.name}`);
                }
            }
        });
    } else {
        // Use DataTransfer interface to access the file(s)
        [...ev.dataTransfer.files].forEach((file, i) => {
            console.log(`… file[${i}].name = ${file.name}`);
            files[i] = createUploadEntry(file)
        });
    }

    handleFiles(files)
}

function handleFiles(files) {
    if (files.length > 0) {
        const submitButton = document.getElementById("submit")
        submitButton.hidden = false
        let itemsWidget = document.getElementById("items");
        itemsWidget.replaceChildren()

        const heading = document.createElement('h5')
        heading.textContent = 'Files to upload (' + files.length + ' items):'
        const list = document.createElement('ul')
        list.className = 'list-group list-group-numbered'

        for (let i = 0; i < files.length; ++i) {
            const item = document.createElement('li')
            item.className = 'list-group-item'
            item.textContent = files[i].displayName
            list.appendChild(item)
        }

        itemsWidget.append(heading, list)

        const area = document.getElementById("area")
        area.dataset.hasFiles = "true"
        dropArea.removeEventListener('click', openFileBrowser)
        document.getElementById("label").hidden = true
        document.getElementById("label2").hidden = true

        submitButton.onclick = function () {
            document.getElementById("processing").hidden = false
            const progress = document.getElementById('progress-bar');
            const data = new FormData();
            let delta = Math.max(1, Math.floor(100 / files.length))
            for (const file of files) {
                data.append('files[]', file.file, file.uploadName);
            }

            const request = new XMLHttpRequest();
            request.open('POST', '/post');
            let i = 0
            const status = document.getElementById("status")
            appendStatusLine(status, 'Load ' + files[i].displayName)
            request.upload.addEventListener('progress', function (e) {
                const percent = Math.floor((e.loaded / e.total) * 100)
                if (files[i] !== undefined && Math.floor(percent / delta) === 0) {
                    i += 1
                    if (files[i] !== undefined) {
                        appendStatusLine(status, 'Load ' + files[i].displayName)
                    }
                }
                progress.style.width = percent + '%';
            })

            request.addEventListener('load', function () {
                console.log(request.status);
                if (request.status < 200 || request.status >= 300) {
                    showUploadError(`Upload failed with status ${request.status}.`)
                    return
                }

                try {
                    createTableFromResponse(request.responseText)
                } catch (error) {
                    console.error(error)
                    showUploadError("The server returned data that could not be rendered.")
                    return
                }

                document.getElementById("area").hidden = true
                document.getElementById("processing").hidden = true;
            })

            request.addEventListener('error', function () {
                showUploadError('Network error while uploading files.')
            })

            submitButton.hidden = true
            request.send(data);
        }
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
                    files.push(createUploadEntry(file))
                    console.log(`… file[${i}].name = ${file.name}`);
                }
            } else if (item.name) {
                files.push({ displayName: item.name })
            }
        });
    } else {
        [...ev.dataTransfer.files].forEach((file, i) => {
            console.log(`… file[${i}].name = ${file.name}`);
            files.push(createUploadEntry(file))
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

function appendStatusLine(container, text) {
    const line = document.createElement('div')
    line.className = 'card-text p-0'
    line.textContent = text
    container.appendChild(line)
}

function showUploadError(message) {
    const status = document.getElementById("status")
    appendStatusLine(status, message)
    document.getElementById("processing").hidden = false
    document.getElementById("submit").hidden = false
}
