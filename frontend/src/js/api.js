import { DIV, TEXT, DOCUMENT, documentCreateElement, insertAt, appendChildren, classListAdd, classListRemove, documentGetElementById, createRepositoryIcon } from "./common.js";
import {
    FLEX, TRUNCATE, ITEMS_CENTER, BORDER, BORDER_NEUTRAL, ROUNDED, TEXT_NEUTRAL, HIDDEN, PY2, SPACE_X_3, BG_NEUTRAL_600, BG_NEUTRAL_100, SHADOW_LG, BG_WHITE, TEXT_CENTER, FONT_MEDIUM, TEXT_2XL, BORDER_B, LIST_NONE, LIST_INSIDE, DIVIDE_Y, PB_1, LIST_GROUP_ITEM, MX_3, SM_MX_2, MX_2, H_12, W_36, SM_W_32, UNDERLINE, W_24, W_28, SM_W_24, W_8, H_8, P_1, W_2, PY_1_5, COLLAPSABLE, P_2, JUSTIFY_START, W_32
} from "./tailwind-classes.js";

function start() {
    const statisticElement = DOCUMENT.querySelector("statistic");
    let api = new Api();

    api.recent().then(data => {
        const id = "recent";
        insertAt(statisticElement, createStatisticBlock("Recent", id), 0);

        let fragment = new RecentList(data).toDocumentFragment();
        appendChildren(documentGetElementById(id), fragment);
    });
    api.popular().then(data => {
        const id = "popular";
        insertAt(statisticElement, createStatisticBlock("Popular", id), 1);
        let fragment = new PopularList(data).toDocumentFragment();
        appendChildren(documentGetElementById(id), fragment);
    });
    api.largest().then(data => {
        const id = "largest";
        insertAt(statisticElement, createStatisticBlock("Largest", id), 2);
        let fragment = new LargestList(data).toDocumentFragment();
        appendChildren(documentGetElementById(id), fragment);
    });
}

function createStatisticBlock(name, id) {
    const block = documentCreateElement(DIV);

    classListAdd(block, BG_WHITE, ROUNDED, BORDER, BORDER_NEUTRAL, `dark:bg-zinc-900`, `dark:border-zinc-500`);
    const elementHeader = documentCreateElement(DIV);
    classListAdd(elementHeader, TEXT_CENTER, FONT_MEDIUM, TEXT_2XL, BORDER_B, PY2, BORDER_NEUTRAL, `dark:border-neutral-600`, `dark:text-neutral-300`, `dark:bg-zinc-800`, 'rounded-t-lg');
    elementHeader.innerText = name;
    const list = documentCreateElement("ul");
    list.id = id;
    classListAdd(list, LIST_NONE, LIST_INSIDE, DIVIDE_Y, PB_1, 'dark:divide-neutral-600');

    appendChildren(block, elementHeader, list);

    return block;
}

class Api {
    constructor() {
        this.url = new URL(DOCUMENT.URL);
    }

    async fetch(url) {
        return await fetch(url)
            .then((response) => { return response.json(); })
            .then((response) => { return response; })
            .catch(function (e) {
                console.log(e);
            });
    }

    async recent() {
        let url = new URL(this.url.protocol + this.url.host + `/api/recent/15`);
        return this.fetch(url);
    }

    async popular() {
        let url = new URL(this.url.protocol + this.url.host + `/api/popular/15`);
        return this.fetch(url);
    }

    async largest() {
        let url = new URL(this.url.protocol + this.url.host + `/api/largest/15`);
        return this.fetch(url);
    }
}

class List {
    constructor(response) {
        this.response = response;
        this.sort_fn = null;
        this.createListItemFn = null;
    }

    toDocumentFragment() {
        let fragment = DOCUMENT.createDocumentFragment();
        this.response.sort(this.sort_fn);
        for (let i = 0; i < this.response.length; ++i) {
            let current = this.response[i];
            let item = this.createListItemFn(current);
            appendChildren(fragment, item.toElement());
        }
        return fragment;
    }
}

class RecentList extends List {
    constructor(response) {
        super(response);
        this.sort_fn = sort_recent;
        this.createListItemFn = (arg) => new RecentListItem(arg);
    }
}

class PopularList extends List {
    constructor(response) {
        super(response);
        this.sort_fn = sort_popular;
        this.createListItemFn = (arg) => new PopularListItem(arg);
    }
}
class LargestList extends List {
    constructor(response) {
        super(response);
        this.sort_fn = sort_largest;
        this.createListItemFn = (arg) => new LargestListItem(arg);
    }
}

function sort_recent(a, b) {
    if (a.time > b.time) return -1;
    if (a.time < b.time) return 1;
    return 0;
}

function sort_popular(a, b) {
    if (a.count > b.count) return -1;
    if (a.count < b.count) return 1;
    return 0;
}

function sort_largest(a, b) {
    if (a.size > b.size) return -1;
    if (a.size < b.size) return 1;
    return 0;
}

class ListItem {
    constructor(repository) {
        this.repository = repository;
        this.id = "";
        this.description = null;
        this.collapse = null;
        if (this.repository.repository_name.slice(-4) === ".git") {
            this.repository.repository_name = this.repository.repository_name.slice(0, -4);
        }
    }

    toElement() {
        let repository = this.repository;
        let repository_array = repository.branches;
        let listItem = documentCreateElement("li");

        classListAdd(listItem, LIST_GROUP_ITEM, MX_3, "sm:mx-1", `dark:text-neutral-300`);

        let local_href = "/" + repository.hostname + "/" + repository.owner + "/" + repository.repository_name;

        let row = createRow(FLEX, MX_2, H_12, ITEMS_CENTER, SPACE_X_3);
        let col1 = createColumn(W_36, SM_W_32, TRUNCATE);

        let title = `${repository.repository_name}`;
        let link = createExternalLink(local_href, repository.repository_name, title, UNDERLINE, TRUNCATE);

        appendChildren(col1, link);
        let col2 = createColumn(W_28, TRUNCATE);

        appendChildren(col2, this.description);

        const href = 'https://' + repository.hostname + "/" + repository.owner + "/" + repository.repository_name;
        const external = createExternalLink(href, "", `Open repository ${href}`);
        const externalLink = createRepositoryIcon(repository.hostname, 16, 16);
        classListAdd(externalLink, BORDER, W_8, H_8, P_1, ROUNDED, `hover:${BG_NEUTRAL_100}`, 'dark:text-white', 'dark:border-neutral-500', 'dark:hover:bg-zinc-800');
        appendChildren(external, externalLink);
        let col4 = createColumn(W_2, PY_1_5);

        appendChildren(row, col1, col2, external, col4);
        if (repository_array.length > 1) {
            let button = createCollapseButton(this.id);
            appendChildren(col4, button);
            let collapse = this.collapse;
            appendChildren(listItem, row, collapse);
        } else {
            appendChildren(listItem, row);
        }
        return listItem;
    }
}

class RecentListItem extends ListItem {
    constructor(response) { super(response); }

    toElement() {
        let repository = this.repository;
        this.id = repository.repository_name + "-recent";

        let now = Date.now();
        let date = Date.parse(repository.time);
        let diff = delta_time(now, date);

        let repository_array = repository.branches;

        if (repository_array.length > 1) {
            this.collapse = new RecentCollapseContent(this.id, repository).toElement();
        }
        this.description = createSmallText(diff);

        return super.toElement();
    }
}

class PopularListItem extends ListItem {
    constructor(response) { super(response); }

    toElement() {
        let repository = this.repository;
        this.id = repository.repository_name + "-popular";
        let branches_array = repository.branches;
        let totalCount = 0;
        for (let i = 0; i < branches_array.length; ++i) { totalCount += branches_array[i].count; }

        if (branches_array.length > 1) {
            this.collapse = new PopularCollapseContent(this.id, repository).toElement();
        }

        this.description = createSmallText(createViewsText(totalCount));

        return super.toElement();
    }
}

class LargestListItem extends ListItem {
    constructor(response) { super(response); }

    toElement() {
        let repository = this.repository;
        this.id = this.repository.repository_name + "-largest";
        let bytes = formatBytes(repository.size);

        let repository_array = repository.branches;
        if (repository_array.length > 1) {
            this.collapse = new LargestCollapseContent(this.id, repository).toElement();
        }
        this.description = createSmallText(bytes);

        return super.toElement();
    }
}

function createExternalLink(href, innerText, title, ...classes) {
    let a = documentCreateElement("a");
    let paragraph = documentCreateElement("p");
    // a.setAttribute("target", "_blank");
    a.setAttribute("rel", "noopener noreferrer canonical");
    a.setAttribute("href", href);
    a.setAttribute("title", title);
    paragraph.innerText = innerText;
    for (let i = 0; i < classes.length; i++) {
        classListAdd(paragraph, classes[i]);
    }
    appendChildren(a, paragraph);
    return a;
}

function createColumn(...classes) {
    let col = documentCreateElement(DIV);
    for (let i = 0; i < classes.length; i++) {
        classListAdd(col, classes[i]);
    }
    return col;
}

function createRow(...classes) {
    let row = documentCreateElement(DIV);
    if (classes.length > 0) {
        classListAdd(row, ...classes);
    }
    return row;
}

class CollapseContent {
    constructor(id, repository) {
        this.id = id;
        this.repository_array = repository.branches;
        this.repository = repository;
        this.elements = [];
    }

    toElement() {
        let div = documentCreateElement(DIV);
        div.id = this.id;
        classListAdd(div, COLLAPSABLE, HIDDEN, P_2);

        for (let i = 0; i < this.repository_array.length; ++i) {
            let row = createRow(FLEX, JUSTIFY_START, SPACE_X_3);

            let col1 = createColumn(W_32, TRUNCATE);

            let text = documentCreateElement(TEXT);
            let branch = this.repository_array[i].branch_name;
            text.innerText = branch;
            text.setAttribute("title", branch);

            appendChildren(col1, text);

            let col2 = createColumn(W_24, TRUNCATE);
            appendChildren(col2, this.elements[i]);
            appendChildren(row, col1, col2);

            appendChildren(div, row);
        }
        return div;
    }
}

class RecentCollapseContent extends CollapseContent {
    constructor(id, repository) {
        super(id, repository);
    }

    toElement() {
        this.repository_array.sort(sort_recent);
        for (let i = 0; i < this.repository_array.length; ++i) {
            let time = this.repository_array[i].time;
            let now = Date.now();
            let date = Date.parse(time);
            let diff = delta_time(now, date);
            let small = createSmallText(diff);
            this.elements[i] = small;
        }
        return super.toElement();
    }
}

class PopularCollapseContent extends CollapseContent {
    constructor(id, repository) {
        super(id, repository);
    }

    toElement() {
        this.repository_array.sort(sort_popular);
        for (let i = 0; i < this.repository_array.length; ++i) {
            let count = this.repository_array[i].count;
            let text = createViewsText(count);
            let small = createSmallText(text);
            this.elements[i] = small;
        }
        return super.toElement();
    }
}

class LargestCollapseContent extends CollapseContent {
    constructor(id, repository) {
        super(id, repository);
    }

    toElement() {
        this.repository_array.sort(sort_largest);
        for (let i = 0; i < this.repository_array.length; ++i) {
            let size = this.repository_array[i].size;
            let bytes = formatBytes(size);
            let small = createSmallText(bytes);
            this.elements[i] = small;
        }
        return super.toElement();
    }
}

function createSmallText(text) {
    let div = documentCreateElement(TEXT);
    classListAdd(div, TEXT_NEUTRAL, 'dark:text-neutral-400');
    div.innerText = text;
    return div;
}

function createCollapseButton(id) {
    let button = documentCreateElement(DIV);
    button.addEventListener("click", function () {
        let element = documentGetElementById(id);
        if (this.classList.toggle("active")) {
            classListRemove(element, HIDDEN);
        } else {
            classListAdd(element, HIDDEN);
        }
    });
    // button.setAttribute("role", "button");
    let html = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-chevron-expand" viewBox="0 0 16 16">
    <path fill-rule="evenodd" d="M3.646 9.146a.5.5 0 0 1 .708 0L8 12.793l3.646-3.647a.5.5 0 0 1 .708.708l-4 4a.5.5 0 0 1-.708 0l-4-4a.5.5 0 0 1 0-.708m0-2.292a.5.5 0 0 0 .708 0L8 3.207l3.646 3.647a.5.5 0 0 0 .708-.708l-4-4a.5.5 0 0 0-.708 0l-4 4a.5.5 0 0 0 0 .708"/>
  </svg>`;

    let tmp = documentCreateElement(DIV);
    tmp.innerHTML = html;
    appendChildren(button, tmp.firstChild);
    return button;
}

function delta_time(now, date) {
    let dt = (now - date) / 1000;
    if (dt > 60 && dt < 7200) {
        dt = Math.round(dt / 60) + " minutes ago";
    }
    else if (dt > 7200 && dt < 86400) {
        dt = Math.round(dt / 3600) + " hours ago";
    }
    else if (dt > 86400) {
        dt = Math.round(dt / 86400) + " days ago";
    }
    else {
        dt = Math.round(dt) + " seconds ago";
    }
    return dt;
}

function createViewsText(count) {
    return count > 1 ? `${count} views` : `${count} view`;
}

function formatBytes(a, b = 2, k = 1024) {
    let d = Math.floor(Math.log(a) / Math.log(k));
    return 0 == a ? "0 Bytes" : parseFloat((a / Math.pow(k, d)).toFixed(Math.max(0, b))) + " " + ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"][d];
}

DOCUMENT.addEventListener("DOMContentLoaded", start);
