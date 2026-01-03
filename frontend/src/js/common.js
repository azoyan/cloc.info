import githubSvg from '../assets/github-icon-1.svg'
import gitlabSvg from '../assets/gitlab-3.svg'
import bitbucketSvg from '../assets/bitbucket-icon.svg'
import codebergSvgWhite from '../assets/codeberg-svgrepo-com.svg'
import codebergSvgDark from '../assets/codeberg-white.svg'
import giteaSvg from '../assets/gitea.svg'
import sourcehutSvg from '../assets/sourcehut.svg'
import gitSvg from '../assets/git-icon.svg'
import gitverseSvg from '../assets/gitverse.svg'
import gitbucketSvg from '../assets/gitbucket.svg'
import gnuSvg from '../assets/gnu.svg'
import launchpadSvg from '../assets/launchpad-logo.svg'

const githubStr = "github.com";
const gitlabStr = "gitlab"
const bitbucketStr = "bitbucket.org"
const giteaStr = "gitea"
const codebergStr = "codeberg.org"
const sourcehutStr = "sr.ht"
const gitverseStr = "gitverse.ru"
const gitbucketStr = "gitbucket"
const gnuStr = ".gnu.org"
const launchpadStr = "launchpad.net"

export const DIV = "div"
export const TEXT = "text"
export const DOCUMENT = document

export let isGithub = (str) => str.includes(githubStr)
export let isGitlab = (str) => str.includes(gitlabStr)
export let isBitbucket = (str) => str.includes(bitbucketStr)
export let isGitea = (str) => str.includes(giteaStr)
export let isCodeberg = (str) => str.includes(codebergStr)
export let isSourcehut = (str) => str.includes(sourcehutStr)
export let isGitverse = (str) => str.includes(gitverseStr)
export let isGitbucket = (str) => str.includes(gitbucketStr)
export let isGnu = (str) => str.includes(gnuStr)
export let isLaunchpad = (str) => str.includes(launchpadStr)


export function createRepositoryIcon(input, width, height) {
    const img = document.createElement('img');
    img.width = width;
    img.height = height;
    img.classList.add("h-6", "text-neutral-100");
    img.alt = "Open repository";
    if (isGithub(input)) {
        const div = document.createElement('div')
        div.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" fill="currentColor"
            viewBox="0 0 16 16">
            <path
                d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8" />
        </svg>`
        return div
        // img.src = githubSvg;
    } else if (isGitlab(input)) {
        img.src = gitlabSvg
    } else if (isBitbucket(input)) {
        img.src = bitbucketSvg
    } else if (isCodeberg(input)) {
        let codebergSvg = `<svg width="22" height="22" viewBox="0 0 4.2333332 4.2333335" version="1.1">
        <defs>
          <linearGradient x1="42519.285" y1="-7078.7891" x2="42575.336" y2="-6966.9307" gradientUnits="userSpaceOnUse" />
          <linearGradient>
            <stop style="stop-color:currentColor;stop-opacity:0" />
            <stop offset="0.49517274" style="stop-color:currentColor;stop-opacity:0.48923996" />
            <stop style="stop-color:currentColor;stop-opacity:0.63279623" />
          </linearGradient>
          <linearGradient xlink:href="#linearGradient6924-6" id="linearGradient6918-3" x1="42519.285" y1="-7078.7891"
            x2="42575.336" y2="-6966.9307" gradientUnits="userSpaceOnUse" />
          <linearGradient id="linearGradient6924-6">
            <stop style="stop-color:currentColor;stop-opacity:0;" />
            <stop offset="0.49517274" style="stop-color:currentColor;stop-opacity:0.30000001;" />
            <stop style="stop-color:currentColor;stop-opacity:0.30000001;" />
          </linearGradient>
        </defs>
        <g transform="matrix(0.06551432,0,0,0.06551432,-2.232417,-1.431776)">
          <path style="fill:url(#linearGradient6918-3);"
            d="m 42519.285,-7078.7891 a 0.76086879,0.56791688 0 0 0 -0.738,0.6739 l 33.586,125.8886 a 87.182358,87.182358 0 0 0 39.381,-33.7636 l -71.565,-92.5196 a 0.76086879,0.56791688 0 0 0 -0.664,-0.2793 z"
            transform="matrix(0.37058478,0,0,0.37058478,-15690.065,2662.0533)" />
          <path style="fill:currentColor"
            d="m 11249.461,-1883.6961 c -12.74,0 -23.067,10.3275 -23.067,23.0671 0,4.3335 1.22,8.5795 3.522,12.2514 l 19.232,-24.8636 c 0.138,-0.1796 0.486,-0.1796 0.624,0 l 19.233,24.8646 c 2.302,-3.6721 3.523,-7.9185 3.523,-12.2524 0,-12.7396 -10.327,-23.0671 -23.067,-23.0671 z"
            transform="matrix(1.4006354,0,0,1.4006354,-15690.065,2662.0533)" />
        </g>
      </svg>
      `
        let div = documentCreateElement("div")
        div.id = "codebergIcon";
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            div.classList.add("text-neutral-500")
            div.classList.remove("text-sky-500")
        }
        else {
            div.classList.add("text-sky-500")
            div.classList.remove("text-neutral-500")
        }

        div.innerHTML = codebergSvg
        window.matchMedia('(prefers-color-scheme: dark)')
            .addEventListener('change', ({ matches }) => {
                if (matches) {
                    div.classList.add("text-neutral-500")
                    div.classList.remove("text-sky-500")
                } else {
                    div.classList.add("text-sky-500")
                    div.classList.remove("text-neutral-500")
                }
            })
        return div
    } else if (isGitea(input)) {
        img.src = giteaSvg
    } else if (isSourcehut(input)) {

        const div = documentCreateElement("div")
        div.innerHTML = `<svg fill="currentColor" height="22" viewBox="0 0 512 512" width="22" xmlns="http://www.w3.org/2000/svg">
        <path
            d="M256 8C119 8 8 119 8 256s111 248 248 248 248-111 248-248S393 8 256 8zm0 448c-110.5 0-200-89.5-200-200S145.5 56 256 56s200 89.5 200 200-89.5 200-200 200z">
        </path>
    </svg>
    `
        return div
    } else if (isGitverse(input)) {
        const div = document.createElement("div")
        div.innerHTML = `<svg version = "1.0" xmlns = "http://www.w3.org/2000/svg" width = "22" height = "22"
        viewBox = "0 0 180.000000 180.000000" preserveAspectRatio = "xMidYMid meet" >
            <g transform="translate(0.000000,180.000000) scale(0.100000,-0.100000)" fill="currentColor" stroke="none">
                <path d="M1463 1745 c-49 -25 -103 -75 -169 -159 l-31 -40 -59 22 c-88 32
    -181 52 -251 52 l-61 0 -36 50 c-41 58 -88 90 -130 90 -46 0 -101 -62 -117
    -134 -12 -55 -15 -59 -56 -77 -127 -54 -269 -176 -347 -297 -90 -138 -126
    -260 -126 -425 0 -141 20 -228 82 -357 46 -95 61 -115 152 -205 83 -83 117
    -109 192 -148 148 -75 221 -92 404 -91 142 0 162 2 240 28 334 109 554 383
    577 718 6 91 -8 209 -33 281 -12 35 -13 68 -3 212 22 328 -21 487 -138 501
    -28 4 -51 -1 -90 -21z m127 -42 c48 -44 62 -110 62 -303 0 -91 -6 -205 -13
    -255 -7 -53 -13 -200 -13 -355 -1 -301 2 -289 -96 -422 -93 -126 -269 -242
    -429 -283 -146 -37 -324 -29 -465 22 -117 42 -116 40 -55 76 30 16 99 64 154
    106 163 124 268 162 466 164 65 1 115 5 113 8 -2 3 -32 12 -66 20 -75 16 -182
    17 -413 5 -152 -8 -177 -7 -225 9 -87 28 -201 91 -255 140 l-50 45 40 -16 c22
    -9 85 -33 140 -52 89 -32 111 -36 201 -36 96 -1 103 0 178 37 43 22 76 40 74
    42 -2 2 -28 -3 -58 -12 -75 -20 -232 -12 -325 17 -113 35 -221 89 -234 117
    -20 43 -12 59 43 102 50 39 65 60 81 117 8 31 -19 54 -63 54 -20 0 -55 6 -77
    12 -40 12 -40 12 50 19 50 4 140 7 200 8 127 1 107 -8 272 128 126 103 172
    123 283 123 l66 0 78 114 c96 142 178 233 231 257 54 25 69 24 105 -8z m-815
    -5 c37 -29 89 -112 131 -209 18 -45 34 -84 34 -89 0 -4 -27 -10 -60 -14 -74
    -9 -121 -27 -185 -75 -85 -63 -79 -69 -71 70 12 209 31 293 75 326 24 18 45
    16 76 -9z m113 -367 c-1 -5 -59 -52 -127 -105 -119 -92 -125 -96 -174 -96 -29
    0 -47 4 -42 9 35 31 226 169 255 183 36 19 94 24 88 9z m-628 -523 c0 -27 -2
    -48 -6 -48 -11 0 -34 71 -34 107 l0 38 20 -25 c13 -17 20 -41 20 -72z" />
                <path d="M1429 1463 c-24 -26 -59 -71 -77 -100 l-33 -52 78 -75 c43 -41 81
    -72 85 -68 15 15 49 218 46 276 -3 57 -5 61 -29 64 -19 2 -36 -8 -70 -45z" />
                <path d="M895 1110 c-22 -43 -19 -68 14 -101 48 -48 120 -37 146 23 18 45 9
    64 -45 87 -69 30 -96 28 -115 -9z" />
            </g>
    </ > `
        return div
    } else if (isGitbucket(input)) {
        img.src = gitbucketSvg
    } else if (isGnu(input)) {
        img.src = gnuSvg
    } else if (isLaunchpad(input)) {
        img.src = launchpadSvg
    }
    else {
        img.src = gitSvg
    }
    return img;
}

export function extractRepositoryHost(input) {
    if (input.includes(githubStr)) {
        return "Github"
    } else if (input.includes(gitlabStr)) {
        return "Gitlab"
    } else if (input.includes(bitbucketStr)) {
        return "Bitbucket"
    } else if (input.includes(codebergStr)) {
        return "Codeberg"
    } else if (input.includes(giteaStr)) {
        return "Gitea"
    } else if (input.includes(sourcehutStr)) {
        return "sourcehut"
    } else if (input.includes(gitverseStr)) {
        return "Gitverse"
    } else if (input.includes(gitbucketStr)) {
        return "Gitbucket"
    } else if (input.includes(gnuStr)) {
        return "Savannah"
    } else if (input.includes(launchpadStr)) {
        return "Launchpad"
    }
    else {
        return "Git"
    }
}

export function createCommitSvgIcon(width, height) {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 100 100");
    svg.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    svg.setAttribute("width", width);
    svg.setAttribute("height", height);

    const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
    circle.setAttribute("cx", "50");
    circle.setAttribute("cy", "50");
    circle.setAttribute("r", "20");
    circle.setAttribute("stroke", "currentColor");
    circle.setAttribute("stroke-width", "5");
    circle.setAttribute("fill", "none");

    const line1 = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line1.setAttribute("x1", "10");
    line1.setAttribute("y1", "50");
    line1.setAttribute("x2", "30");
    line1.setAttribute("y2", "50");
    line1.setAttribute("stroke", "currentColor");
    line1.setAttribute("stroke-width", "5");

    const line2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line2.setAttribute("x1", "70");
    line2.setAttribute("y1", "50");
    line2.setAttribute("x2", "90");
    line2.setAttribute("y2", "50");
    line2.setAttribute("stroke", "currentColor");
    line2.setAttribute("stroke-width", "5");

    svg.appendChild(circle);
    svg.appendChild(line1);
    svg.appendChild(line2);

    return svg;
}

export function createTableFromResponse(data) {
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

    // document.getElementById('toggleButton').addEventListener('click', function () {
    //   var collapsibleDiv = document.getElementById('collapsibleDiv');
    //   if (collapsibleDiv.style.display === 'none') {
    //     collapsibleDiv.style.display = 'block';
    //     setTimeout(function () {
    //       collapsibleDiv.style.transform = 'scaleY(1)';
    //     }, 20);
    //   } else {
    //     collapsibleDiv.style.transform = 'scaleY(0)';
    //     setTimeout(function () {
    //       collapsibleDiv.style.display = 'none';
    //     }, 10);
    //   }
    // });
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

    let table = '<table class="table-auto dark:text-white">'
    table += createTableHead(strings[0])
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

export function createTableHead(array) {
    let thead = '<thead><tr>'
    for (let i = 0; i < array.length; ++i) {
        thead += '<th scope="col">' + array[i] + '</th>'
    }
    thead += "</tr></thead>"
    return thead;
}

export function createTableRow(array) {
    let row = "<tr>"

    row += '<th>' + array[0] + '</th>'

    for (let i = 1; i < array.length; ++i) {
        let val = array[i];
        const num = Number(val);
        if (!isNaN(num)) {
            val = num.toLocaleString()
        }
        row += "<td>" + val + "</td>"
    }
    row += "</tr>"
    return row
}

export function createCocomoFromResponse(cocomo_data) {
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

export function documentGetElementById(id) {
    return document.getElementById(id)
}
export function log(...data) {
    console.log(data)
}


export function swapElements(repoInfo, invalidFeedback) {
    const parent = repoInfo.parentNode;
    parent.insertBefore(repoInfo, invalidFeedback);
    parent.insertBefore(invalidFeedback, repoInfo);
}

export function disable(el) {
    el.setAttribute("disabled", "")
}

export function classListAdd(element, ...classes) {
    for (let i = 0; i < classes.length; i++) {
        element.classList.add(classes[i])
    }
}

export function classListRemove(element, ...classes) {
    for (let i = 0; i < classes.length; i++) {
        element.classList.remove(classes[i])
    }
}

export function appendChildren(parent, ...children) {
    for (let i = 0; i < children.length; ++i) {
        parent.appendChild(children[i])
    }
}
export function insertAt(parentElement, childElement, index) {
    if (index >= parentElement.children.length) {
        parentElement.appendChild(childElement);
    } else {
        parentElement.insertBefore(childElement, parentElement.children[index]);
    }
}

export function documentCreateElement(tagName) {
    return document.createElement(tagName)
}
