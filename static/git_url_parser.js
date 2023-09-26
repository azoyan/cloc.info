function extractBranchFromGitUrl(git_url) {
    let branch_word

    if (git_url.host === "gitlab.com" || git_url.host === "github.com") {
        branch_word = "tree"
    }
    else if (git_url.host === "bitbucket.org") {
        branch_word = "src"
    }
    else if (git_url.host === "codeberg.org") {
        branch_word = "src/branch"
    }

    let branch_word_idx = git_url.pathname.indexOf(branch_word)
    if (git_url.owner === branch_word) {
        branch_word_idx = git_url.pathname.indexOf(branch_word, branch_word_idx + branch_word.length)
    }
    if (git_url.name === branch_word) {
        branch_word_idx = git_url.pathname.indexOf(branch_word, branch_word_idx + branch_word.length)
    }

    console.log(git_url.pathname, "branch_word", branch_word, branch_word_idx)
    return branch_word_idx > 0 ? git_url.pathname.slice(branch_word_idx + branch_word.length + 1) : undefined
}
function gitUrlParse(url) {
    if (typeof url !== "string") {
        throw new Error("The url must be a string.");
    }

    let urlInfo = gitUp(url)
        , sourceParts = urlInfo.resource.split(".")
        , splits = null
        ;

    urlInfo.toString = function (type) {
        return gitUrlParse.stringify(this, type);
    };

    urlInfo.source = sourceParts.length > 2
        ? sourceParts.slice(1 - sourceParts.length).join(".")
        : urlInfo.source = urlInfo.resource
        ;

    // Note: Some hosting services (e.g. Visual Studio Team Services) allow whitespace characters
    // in the repository and owner names so we decode the URL pieces to get the correct result
    urlInfo.git_suffix = /\.git$/.test(urlInfo.pathname);
    urlInfo.name = decodeURIComponent((urlInfo.pathname || urlInfo.href).replace(/(^\/)|(\/$)/g, '').replace(/\.git$/, ""));
    urlInfo.owner = decodeURIComponent(urlInfo.user);

    switch (urlInfo.source) {
        case "git.cloudforge.com":
            urlInfo.owner = urlInfo.user;
            urlInfo.organization = sourceParts[0];
            urlInfo.source = "cloudforge.com";
            break;
        case "visualstudio.com":
            // Handle VSTS SSH URLs
            if (urlInfo.resource === 'vs-ssh.visualstudio.com') {
                splits = urlInfo.name.split("/");
                if (splits.length === 4) {
                    urlInfo.organization = splits[1];
                    urlInfo.owner = splits[2];
                    urlInfo.name = splits[3];
                    urlInfo.full_name = splits[2] + '/' + splits[3];
                }
                break;
            } else {
                splits = urlInfo.name.split("/");
                if (splits.length === 2) {
                    urlInfo.owner = splits[1];
                    urlInfo.name = splits[1];
                    urlInfo.full_name = '_git/' + urlInfo.name;
                } else if (splits.length === 3) {
                    urlInfo.name = splits[2];
                    if (splits[0] === 'DefaultCollection') {
                        urlInfo.owner = splits[2];
                        urlInfo.organization = splits[0];
                        urlInfo.full_name = urlInfo.organization + '/_git/' + urlInfo.name;
                    } else {
                        urlInfo.owner = splits[0];
                        urlInfo.full_name = urlInfo.owner + '/_git/' + urlInfo.name;
                    }
                } else if (splits.length === 4) {
                    urlInfo.organization = splits[0];
                    urlInfo.owner = splits[1];
                    urlInfo.name = splits[3];
                    urlInfo.full_name = urlInfo.organization + '/' + urlInfo.owner + '/_git/' + urlInfo.name;
                }
                break;
            }

        // Azure DevOps (formerly Visual Studio Team Services)
        case "dev.azure.com":
        case "azure.com":
            if (urlInfo.resource === 'ssh.dev.azure.com') {
                splits = urlInfo.name.split("/");
                if (splits.length === 4) {
                    urlInfo.organization = splits[1];
                    urlInfo.owner = splits[2];
                    urlInfo.name = splits[3];
                }
                break;
            } else {
                splits = urlInfo.name.split("/");
                if (splits.length === 5) {
                    urlInfo.organization = splits[0];
                    urlInfo.owner = splits[1];
                    urlInfo.name = splits[4];
                    urlInfo.full_name = '_git/' + urlInfo.name;
                } else if (splits.length === 3) {
                    urlInfo.name = splits[2];
                    if (splits[0] === 'DefaultCollection') {
                        urlInfo.owner = splits[2];
                        urlInfo.organization = splits[0];
                        urlInfo.full_name = urlInfo.organization + '/_git/' + urlInfo.name;
                    } else {
                        urlInfo.owner = splits[0];
                        urlInfo.full_name = urlInfo.owner + '/_git/' + urlInfo.name;
                    }
                } else if (splits.length === 4) {
                    urlInfo.organization = splits[0];
                    urlInfo.owner = splits[1];
                    urlInfo.name = splits[3];
                    urlInfo.full_name = urlInfo.organization + '/' + urlInfo.owner + '/_git/' + urlInfo.name;
                }
                if (urlInfo.query && urlInfo.query['path']) {
                    urlInfo.filepath = urlInfo.query['path'].replace(/^\/+/g, ''); // Strip leading slash (/)
                }
                if (urlInfo.query && urlInfo.query['version']) {  // version=GB<branch>
                    urlInfo.ref = urlInfo.query['version'].replace(/^GB/, ''); // remove GB
                }
                break;
            }
        default:
            splits = urlInfo.name.split("/");
            let nameIndex = splits.length - 1;
            if (splits.length >= 2) {
                const dashIndex = splits.indexOf("-", 2)
                const blobIndex = splits.indexOf("blob", 2);
                const treeIndex = splits.indexOf("tree", 2);
                const commitIndex = splits.indexOf("commit", 2);
                const srcIndex = splits.indexOf("src", 2);
                const rawIndex = splits.indexOf("raw", 2);
                nameIndex = dashIndex > 0 ? dashIndex - 1
                    : blobIndex > 0 ? blobIndex - 1
                        : treeIndex > 0 ? treeIndex - 1
                            : commitIndex > 0 ? commitIndex - 1
                                : srcIndex > 0 ? srcIndex - 1
                                    : rawIndex > 0 ? rawIndex - 1
                                        : nameIndex;

                urlInfo.owner = splits.slice(0, nameIndex).join('/');
                urlInfo.name = splits[nameIndex];
                if (commitIndex) {
                    urlInfo.commit = splits[nameIndex + 2]
                }
            }

            urlInfo.ref = "";
            urlInfo.filepathtype = "";
            urlInfo.filepath = "";
            const offsetNameIndex = splits.length > nameIndex && splits[nameIndex + 1] === "-" ? nameIndex + 1 : nameIndex;
            if ((splits.length > offsetNameIndex + 2) && (["raw", "src", "blob", "tree"].indexOf(splits[offsetNameIndex + 1]) >= 0)) {
                urlInfo.filepathtype = splits[offsetNameIndex + 1];
                urlInfo.ref = splits[offsetNameIndex + 2];
                if (splits.length > offsetNameIndex + 3) {
                    urlInfo.filepath = splits.slice(offsetNameIndex + 3).join('/');
                }
            }
            urlInfo.organization = urlInfo.owner;
            break;
    }

    if (!urlInfo.full_name) {
        urlInfo.full_name = urlInfo.owner;
        if (urlInfo.name) {
            urlInfo.full_name && (urlInfo.full_name += "/");
            urlInfo.full_name += urlInfo.name;
        }
    }
    // Bitbucket Server
    if (urlInfo.owner.startsWith("scm/")) {
        urlInfo.source = "bitbucket-server";
        urlInfo.owner = urlInfo.owner.replace("scm/", "");
        urlInfo.organization = urlInfo.owner;
        urlInfo.full_name = `${urlInfo.owner}/${urlInfo.name}`
    }

    const bitbucket = /(projects|users)\/(.*?)\/repos\/(.*?)((\/.*$)|$)/
    const matches = bitbucket.exec(urlInfo.pathname)
    if (matches != null) {
        urlInfo.source = "bitbucket-server";
        if (matches[1] === "users") {
            urlInfo.owner = "~" + matches[2];
        } else {
            urlInfo.owner = matches[2];
        }

        urlInfo.organization = urlInfo.owner;
        urlInfo.name = matches[3];

        splits = matches[4].split("/");
        if (splits.length > 1) {
            if (["raw", "browse"].indexOf(splits[1]) >= 0) {
                urlInfo.filepathtype = splits[1];
                if (splits.length > 2) {
                    urlInfo.filepath = splits.slice(2).join('/');
                }
            } else if (splits[1] === "commits" && splits.length > 2) {
                urlInfo.commit = splits[2];
            }
        }
        urlInfo.full_name = `${urlInfo.owner}/${urlInfo.name}`

        if (urlInfo.query.at) {
            urlInfo.ref = urlInfo.query.at;
        } else {
            urlInfo.ref = "";
        }
    }
    return urlInfo;
}

/**
 * stringify
 * Stringifies a `GitUrl` object.
 *
 * @name stringify
 * @function
 * @param {GitUrl} obj The parsed Git url object.
 * @param {String} type The type of the stringified url (default `obj.protocol`).
 * @return {String} The stringified url.
 */
gitUrlParse.stringify = function (obj, type) {
    type = type || ((obj.protocols && obj.protocols.length) ? obj.protocols.join('+') : obj.protocol);
    const port = obj.port ? `:${obj.port}` : '';
    const user = obj.user || 'git';
    const maybeGitSuffix = obj.git_suffix ? ".git" : ""
    switch (type) {
        case "ssh":
            if (port)
                return `ssh://${user}@${obj.resource}${port}/${obj.full_name}${maybeGitSuffix}`;
            else
                return `${user}@${obj.resource}:${obj.full_name}${maybeGitSuffix}`;
        case "git+ssh":
        case "ssh+git":
        case "ftp":
        case "ftps":
            return `${type}://${user}@${obj.resource}${port}/${obj.full_name}${maybeGitSuffix}`;
        case "http":
        case "https":
            const auth = obj.token
                ? buildToken(obj) : obj.user && (obj.protocols.includes('http') || obj.protocols.includes('https'))
                    ? `${obj.user}@` : "";
            return `${type}://${auth}${obj.resource}${port}/${buildPath(obj)}${maybeGitSuffix}`;
        default:
            return obj.href;
    }
};

/*!
 * buildToken
 * Builds OAuth token prefix (helper function)
 *
 * @name buildToken
 * @function
 * @param {GitUrl} obj The parsed Git url object.
 * @return {String} token prefix
 */
function buildToken(obj) {
    switch (obj.source) {
        case "bitbucket.org":
            return `x-token-auth:${obj.token}@`;
        default:
            return `${obj.token}@`
    }
}

function buildPath(obj) {
    switch (obj.source) {
        case "bitbucket-server":
            return `scm/${obj.full_name}`;
        default:
            return `${obj.full_name}`;

    }
}

function gitUp(input) {
    let output = parseUrl(input);
    output.token = "";

    if (output.password === "x-oauth-basic") {
        output.token = output.user;
    } else if (output.user === "x-token-auth") {
        output.token = output.password
    }

    if (isSsh(output.protocols) || (output.protocols.length === 0 && isSsh(input))) {
        output.protocol = "ssh";
    } else if (output.protocols.length) {
        output.protocol = output.protocols[0];
    } else {
        output.protocol = "file";
        output.protocols = ["file"]
    }

    output.href = output.href.replace(/\/$/, "")
    return output;
}

function isSsh(input) {

    if (Array.isArray(input)) {
        return input.indexOf("ssh") !== -1 || input.indexOf("rsync") !== -1;
    }

    if (typeof input !== "string") {
        return false;
    }

    const prots = protocols(input);
    input = input.substring(input.indexOf("://") + 3);
    if (isSsh(prots)) {
        return true;
    }

    // TODO This probably could be improved :)
    const urlPortPattern = new RegExp('\.([a-zA-Z\\d]+):(\\d+)\/');
    return !input.match(urlPortPattern) && input.indexOf("@") < input.indexOf(":");
}



function _interopDefaultLegacy(e) { return e && typeof e === 'object' && 'default' in e ? e : { 'default': e }; }

var parsePath__default = /*#__PURE__*/_interopDefaultLegacy(parsePath);
var normalizeUrl__default = /*#__PURE__*/_interopDefaultLegacy(normalizeUrl);

// Dependencies

/**
 * parseUrl
 * Parses the input url.
 *
 * **Note**: This *throws* if invalid urls are provided.
 *
 * @name parseUrl
 * @function
 * @param {String} url The input url.
 * @param {Boolean|Object} normalize Whether to normalize the url or not.
 *                         Default is `false`. If `true`, the url will
 *                         be normalized. If an object, it will be the
 *                         options object sent to [`normalize-url`](https://github.com/sindresorhus/normalize-url).
 *
 *                         For SSH urls, normalize won't work.
 *
 * @return {Object} An object containing the following fields:
 *
 *    - `protocols` (Array): An array with the url protocols (usually it has one element).
 *    - `protocol` (String): The first protocol, `"ssh"` (if the url is a ssh url) or `"file"`.
 *    - `port` (null|Number): The domain port.
 *    - `resource` (String): The url domain (including subdomains).
 *    - `user` (String): The authentication user (usually for ssh urls).
 *    - `pathname` (String): The url pathname.
 *    - `hash` (String): The url hash.
 *    - `search` (String): The url querystring value.
 *    - `href` (String): The input url.
 *    - `query` (Object): The url querystring, parsed as object.
 *    - `parse_failed` (Boolean): Whether the parsing failed or not.
 */
const parseUrl = (url, normalize = false) => {

    // Constants
    const GIT_RE = /(^(git@|http(s)?:\/\/)([\w\.\-@]+)(\/|:))(([\~,\.\w,\-,\_,\/]+)(.git){0,1}((\/){0,1}))/;

    const throwErr = msg => {
        const err = new Error(msg);
        err.subject_url = url;
        throw err
    };

    if (typeof url !== "string" || !url.trim()) {
        throwErr("Invalid url.");
    }

    if (url.length > parseUrl.MAX_INPUT_LENGTH) {
        throwErr("Input exceeds maximum length. If needed, change the value of parseUrl.MAX_INPUT_LENGTH.");
    }

    if (normalize) {
        if (typeof normalize !== "object") {
            normalize = {
                stripHash: false
            };
        }
        url = normalizeUrl__default["default"](url, normalize);
    }

    const parsed = parsePath__default["default"](url);

    // Potential git-ssh urls
    if (parsed.parse_failed) {
        const matched = parsed.href.match(GIT_RE);
        if (matched) {
            parsed.protocols = ["ssh"];
            parsed.protocol = "ssh";
            parsed.resource = matched[4];
            parsed.host = matched[4];
            parsed.user = "git";
            parsed.pathname = `/${matched[6]}`;
            parsed.parse_failed = false;
        } else {
            throwErr("URL parsing failed.");
        }
    }

    return parsed;
};

parseUrl.MAX_INPUT_LENGTH = 2048;

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs
const DATA_URL_DEFAULT_MIME_TYPE = 'text/plain';
const DATA_URL_DEFAULT_CHARSET = 'us-ascii';

const testParameter = (name, filters) => filters.some(filter => filter instanceof RegExp ? filter.test(name) : filter === name);

const normalizeDataURL = (urlString, { stripHash }) => {
    const match = /^data:(?<type>[^,]*?),(?<data>[^#]*?)(?:#(?<hash>.*))?$/.exec(urlString);

    if (!match) {
        throw new Error(`Invalid URL: ${urlString}`);
    }

    let { type, data, hash } = match.groups;
    const mediaType = type.split(';');
    hash = stripHash ? '' : hash;

    let isBase64 = false;
    if (mediaType[mediaType.length - 1] === 'base64') {
        mediaType.pop();
        isBase64 = true;
    }

    // Lowercase MIME type
    const mimeType = (mediaType.shift() || '').toLowerCase();
    const attributes = mediaType
        .map(attribute => {
            let [key, value = ''] = attribute.split('=').map(string => string.trim());

            // Lowercase `charset`
            if (key === 'charset') {
                value = value.toLowerCase();

                if (value === DATA_URL_DEFAULT_CHARSET) {
                    return '';
                }
            }

            return `${key}${value ? `=${value}` : ''}`;
        })
        .filter(Boolean);

    const normalizedMediaType = [
        ...attributes,
    ];

    if (isBase64) {
        normalizedMediaType.push('base64');
    }

    if (normalizedMediaType.length > 0 || (mimeType && mimeType !== DATA_URL_DEFAULT_MIME_TYPE)) {
        normalizedMediaType.unshift(mimeType);
    }

    return `data:${normalizedMediaType.join(';')},${isBase64 ? data.trim() : data}${hash ? `#${hash}` : ''}`;
};

function normalizeUrl(urlString, options) {
    options = {
        defaultProtocol: 'http:',
        normalizeProtocol: true,
        forceHttp: false,
        forceHttps: false,
        stripAuthentication: true,
        stripHash: false,
        stripTextFragment: true,
        stripWWW: true,
        removeQueryParameters: [/^utm_\w+/i],
        removeTrailingSlash: true,
        removeSingleSlash: true,
        removeDirectoryIndex: false,
        sortQueryParameters: true,
        ...options,
    };

    urlString = urlString.trim();

    // Data URL
    if (/^data:/i.test(urlString)) {
        return normalizeDataURL(urlString, options);
    }

    if (/^view-source:/i.test(urlString)) {
        throw new Error('`view-source:` is not supported as it is a non-standard protocol');
    }

    const hasRelativeProtocol = urlString.startsWith('//');
    const isRelativeUrl = !hasRelativeProtocol && /^\.*\//.test(urlString);

    // Prepend protocol
    if (!isRelativeUrl) {
        urlString = urlString.replace(/^(?!(?:\w+:)?\/\/)|^\/\//, options.defaultProtocol);
    }

    const urlObject = new URL(urlString);

    if (options.forceHttp && options.forceHttps) {
        throw new Error('The `forceHttp` and `forceHttps` options cannot be used together');
    }

    if (options.forceHttp && urlObject.protocol === 'https:') {
        urlObject.protocol = 'http:';
    }

    if (options.forceHttps && urlObject.protocol === 'http:') {
        urlObject.protocol = 'https:';
    }

    // Remove auth
    if (options.stripAuthentication) {
        urlObject.username = '';
        urlObject.password = '';
    }

    // Remove hash
    if (options.stripHash) {
        urlObject.hash = '';
    } else if (options.stripTextFragment) {
        urlObject.hash = urlObject.hash.replace(/#?:~:text.*?$/i, '');
    }

    // Remove duplicate slashes if not preceded by a protocol
    // NOTE: This could be implemented using a single negative lookbehind
    // regex, but we avoid that to maintain compatibility with older js engines
    // which do not have support for that feature.
    if (urlObject.pathname) {
        // TODO: Replace everything below with `urlObject.pathname = urlObject.pathname.replace(/(?<!\b[a-z][a-z\d+\-.]{1,50}:)\/{2,}/g, '/');` when Safari supports negative lookbehind.

        // Split the string by occurrences of this protocol regex, and perform
        // duplicate-slash replacement on the strings between those occurrences
        // (if any).
        const protocolRegex = /\b[a-z][a-z\d+\-.]{1,50}:\/\//g;

        let lastIndex = 0;
        let result = '';
        for (; ;) {
            const match = protocolRegex.exec(urlObject.pathname);
            if (!match) {
                break;
            }

            const protocol = match[0];
            const protocolAtIndex = match.index;
            const intermediate = urlObject.pathname.slice(lastIndex, protocolAtIndex);

            result += intermediate.replace(/\/{2,}/g, '/');
            result += protocol;
            lastIndex = protocolAtIndex + protocol.length;
        }

        const remnant = urlObject.pathname.slice(lastIndex, urlObject.pathname.length);
        result += remnant.replace(/\/{2,}/g, '/');

        urlObject.pathname = result;
    }

    // Decode URI octets
    if (urlObject.pathname) {
        try {
            urlObject.pathname = decodeURI(urlObject.pathname);
        } catch { }
    }

    // Remove directory index
    if (options.removeDirectoryIndex === true) {
        options.removeDirectoryIndex = [/^index\.[a-z]+$/];
    }

    if (Array.isArray(options.removeDirectoryIndex) && options.removeDirectoryIndex.length > 0) {
        let pathComponents = urlObject.pathname.split('/');
        const lastComponent = pathComponents[pathComponents.length - 1];

        if (testParameter(lastComponent, options.removeDirectoryIndex)) {
            pathComponents = pathComponents.slice(0, -1);
            urlObject.pathname = pathComponents.slice(1).join('/') + '/';
        }
    }

    if (urlObject.hostname) {
        // Remove trailing dot
        urlObject.hostname = urlObject.hostname.replace(/\.$/, '');

        // Remove `www.`
        if (options.stripWWW && /^www\.(?!www\.)[a-z\-\d]{1,63}\.[a-z.\-\d]{2,63}$/.test(urlObject.hostname)) {
            // Each label should be max 63 at length (min: 1).
            // Source: https://en.wikipedia.org/wiki/Hostname#Restrictions_on_valid_host_names
            // Each TLD should be up to 63 characters long (min: 2).
            // It is technically possible to have a single character TLD, but none currently exist.
            urlObject.hostname = urlObject.hostname.replace(/^www\./, '');
        }
    }

    // Remove query unwanted parameters
    if (Array.isArray(options.removeQueryParameters)) {
        // eslint-disable-next-line unicorn/no-useless-spread -- We are intentionally spreading to get a copy.
        for (const key of [...urlObject.searchParams.keys()]) {
            if (testParameter(key, options.removeQueryParameters)) {
                urlObject.searchParams.delete(key);
            }
        }
    }

    if (options.removeQueryParameters === true) {
        urlObject.search = '';
    }

    // Sort query parameters
    if (options.sortQueryParameters) {
        urlObject.searchParams.sort();

        // Calling `.sort()` encodes the search parameters, so we need to decode them again.
        try {
            urlObject.search = decodeURIComponent(urlObject.search);
        } catch { }
    }

    if (options.removeTrailingSlash) {
        urlObject.pathname = urlObject.pathname.replace(/\/$/, '');
    }

    const oldUrlString = urlString;

    // Take advantage of many of the Node `url` normalizations
    urlString = urlObject.toString();

    if (!options.removeSingleSlash && urlObject.pathname === '/' && !oldUrlString.endsWith('/') && urlObject.hash === '') {
        urlString = urlString.replace(/\/$/, '');
    }

    // Remove ending `/` unless removeSingleSlash is false
    if ((options.removeTrailingSlash || urlObject.pathname === '/') && urlObject.hash === '' && options.removeSingleSlash) {
        urlString = urlString.replace(/\/$/, '');
    }

    // Restore relative protocol, if applicable
    if (hasRelativeProtocol && !options.normalizeProtocol) {
        urlString = urlString.replace(/^http:\/\//, '//');
    }

    // Remove http/https
    if (options.stripProtocol) {
        urlString = urlString.replace(/^(?:https?:)?\/\//, '');
    }

    return urlString;
}

function parsePath(url) {

    const output = {
        protocols: []
        , protocol: null
        , port: null
        , resource: ""
        , host: ""
        , user: ""
        , password: ""
        , pathname: ""
        , hash: ""
        , search: ""
        , href: url
        , query: {}
        , parse_failed: false
    }

    try {
        const parsed = new URL(url)
        output.protocols = protocols(parsed)
        output.protocol = output.protocols[0]
        output.port = parsed.port
        output.resource = parsed.hostname
        output.host = parsed.host
        output.user = parsed.username || ""
        output.password = parsed.password || ""
        output.pathname = parsed.pathname
        output.hash = parsed.hash.slice(1)
        output.search = parsed.search.slice(1)
        output.href = parsed.href
        output.query = Object.fromEntries(parsed.searchParams)
    } catch (e) {
        // TODO Maybe check if it is a valid local file path
        //      In any case, these will be parsed by higher
        //      level parsers such as parse-url, git-url-parse, git-up
        output.protocols = ["file"]
        output.protocol = output.protocols[0]
        output.port = ""
        output.resource = ""
        output.user = ""
        output.pathname = ""
        output.hash = ""
        output.search = ""
        output.href = url
        output.query = {}
        output.parse_failed = true
    }

    return output;
}

function protocols(input, first) {
    if (first === true) {
        first = 0;
    }

    let prots = ""
    if (typeof input === "string") {
        try {
            prots = new URL(input).protocol
        } catch (e) { }
    } else if (input && input.constructor === URL) {
        prots = input.protocol
    }

    const splits = prots.split(/\:|\+/).filter(Boolean)

    if (typeof first === "number") {
        return splits[first];
    }

    return splits;
};