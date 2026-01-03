import later from "./later.js"

import { DIV, TEXT, DOCUMENT, createRepositoryIcon, createCommitSvgIcon, extractRepositoryHost, appendChildren, classListAdd, classListRemove, disable, swapElements, documentGetElementById, documentCreateElement } from "./common.js"
import {
  FLEX, TRUNCATE, ITEMS_CENTER, BORDER, BORDER_NEUTRAL, BORDER_R, ROUNDED, TEXT_NEUTRAL_600, HIDDEN, PX_2, PY_2, W_2, TEXT_SM, BORDER_RED, FOCUS_RING, BG_NEUTRAL_100, SM, MD, BLOCK, INVISIBLE, SPACE_X_3, CURSOR_POINTER, HOVER_TEXT_BLACK, PX_1, RING_1, RING_INSET, RING_GRAY_500_10, ROUNDED_L_LG, SM_PL_2, SM_PR_4, HOVER_ROUNDED, FONT_MONO, FONT_LIGHT,
  DARK_TEXT_NEUTRAL_300,
  DARK_BORDER_ZINC_500,
  DARK_BG_ZINC_900,
  DARK_HOVER_TEXT_NEUTRAL_100,
  ROUNDED_LG,
  DARK_TEXT_NEUTRAL_200,
  DARK_BG_ZINC_800,
  MD_BLOCK,
  SM_BLOCK,
  DARK_HOVER_ROUNDED_LG
} from './tailwind-classes.js'
import { gitUrlParse, extractBranchFromGitUrl } from "./git_url_parser.js"
import { log } from "./common.js"
const DEFAULT = "default"
const COMMIT = "Commit"
const LAST_COMMIT_SHA = "Last commit SHA"

const input = documentGetElementById('input')
const commit = documentGetElementById("commit")
const invalidFeedback = documentGetElementById('invalidFeedback')
const repositoryInfo = documentGetElementById("repoInfo")
const hint = documentGetElementById("hint")
const hintUrl = documentGetElementById("hintUrl")
const checkMark = documentGetElementById("check")
const dropdownButton = documentGetElementById("dropdownButton")
const repoUrl = documentGetElementById("repoUrl")
const dropdownList = documentGetElementById("dropdownList")
const branchLabel = documentGetElementById("branchLabel")
const submitButton = documentGetElementById("submitButton")
const buttonText = documentGetElementById("buttonText")
const submitText = "Count"

let Later = null
let Visibility = 0
let Branches

submitButton.onclick = function () {
  const url = gitUrlParse(input.value)
  log(branchLabel.value)
  const selected = branchLabel.innerText;
  // log("SubmitButton onclick()", selected, url)
  let path = url.host + '/' + url.owner + '/' + url.name;
  if (selected !== Branches.default_branch) {
    if (url.host === "github.com" || url.host === "gitlab.com") {
      path += '/tree/'
    } else if (url.host === "bitbucket.org") {
      path += '/src/'
    }
    else if (url.host === "codeberg.org") {
      path += "/src/branch/"
    }
    else if (url.host === "gitea.com") {
      path += "/src/branch/"
    }
    path += selected;
  }

  path = path.replace(/\/+$/g, '')
  log("path", path);
  window.location.href = path
}

hintUrl.onclick = function () {
  input.value = hintUrl.innerText
  check(input.value);
}

input.oninput = (evt) => {
  const isPasted = evt.inputType && evt.inputType.startsWith("insertFromPaste");
  const value = evt.target.value
  if (isPasted) {
    pasteValue(value)
  } else {
    if (value === "") {
      reset()
    }
    else {
      editValue(value)
    }
  }
}

input.onkeydown = (e) => { if (e.key === ' ') e.preventDefault(); }

function reset() {
  cancelLaterTimer()
  // log("reset")
  setVisible(false, repositoryInfo, invalidFeedback, checkMark)
  setVisible(true, hint)
  classListAdd(input, FOCUS_RING)
  classListRemove(input, BORDER_RED)
  disable(submitButton)

  appendChildren(invalidFeedback.parentElement, invalidFeedback)
}

function setVisible(needVisible, ...elements) {
  for (let i = 0; i < elements.length; i++) {
    if (needVisible) {
      classListRemove(elements[i], INVISIBLE)
    } else {
      classListAdd(elements[i], INVISIBLE)
    }
  }
}

function cancelLaterTimer() {
  if (Later != null) {
    Later.cancel();
    Later = null;
  }
}

async function editValue(value) {
  cancelLaterTimer()
  setVisible(false, repositoryInfo, checkMark)
  // log("edit", value)
  if (Later === null) {
    Later = later.later(2000, false)
    Later.promise
      .then(function () {
        // log(value)
        check(value)
      })
      .catch(() => {
        // log("later cancelled", e); 
      });
  }
}

function pasteValue(value) {
  if (value === "") return;
  check(value)
}

function check(urlStr) {
  urlStr = urlStr.replace(/\/+$/g, '')

  // let git_extension = urlStr.slice(-4);
  // if (git_extension !== ".git") {
  //     url_str += ".git"
  // }

  // let is_git_regex = /(?:git|ssh|https?|git@[-\w.]+):(\/\/)?(.*?)(\.git)(\/?|\#[-\d\w._]+?)$/;
  // if (!url_str.match(is_git_regex)) { url_str = 'https://' + url_str; }
  // log(url_str);
  let parsed_url
  try {
    parsed_url = gitUrlParse(urlStr)
  }
  catch (error) {
    showError(notValidUrlText(urlStr))
    return
  }
  // log("parsed:", parsed_url)
  // reset()
  if (parsed_url.parse_failed) {
    showError(notValidUrlText(urlStr))
    return
  }

  buttonText.innerText = "Checking..."
  disable(submitButton)
  setVisible(false, repositoryInfo, checkMark)
  classListRemove(checkSpinner, HIDDEN);

  const repository_name = parsed_url.name
  // if (repository_name.slice(-4) !== ".git") { repository_name += ".git" }
  let branches_api = DOCUMENT.URL + "api/" + parsed_url.host + '/' + parsed_url.owner + '/' + repository_name + "/branches";
  branches_api = branches_api.replace(/([^:]\/)\/+/g, "$1");
  const current_branch = extractBranchFromGitUrl(parsed_url)

  // log("branches_api", branches_api, parsed_url.toString())
  // log("current_branch", current_branch)

  fetch(branches_api)
    .then((response) => response.json())
    .then((response) => {
      Branches = response;
      updateRepositoryPicture()
      updateSelect(Branches, current_branch);
      updateCommitLabel()

      submitButton.removeAttribute("disabled");

      DOCUMENT.addEventListener("keypress", function (event) {
        if (event.key === "Enter") {
          event.preventDefault();
          submitButton.click();
        }
      });
      setVisible(true, repositoryInfo, checkMark)
      dropdownButton.onclick = () => { Visibility = 1, setVisible(true, dropdownList) }
      setVisible(false, hint, invalidFeedback)

      classListRemove(input, BORDER_RED);
      classListAdd(input, FOCUS_RING)
      appendChildren(invalidFeedback.parentElement, invalidFeedback)
      classListAdd(checkSpinner, HIDDEN);
      buttonText.innerText = submitText
    })
    .catch(function (error) {
      if (error.response) {
        showError(error.response.data)
        console.error(error.response.data);
        console.error(error.response.status);
        console.error(error.response.headers);
      }
      else {
        log("error", error)
        showError(notValidUrlText(urlStr))
      }
    });
}
function showError(errorText) {
  setVisible(true, invalidFeedback)
  invalidFeedback.innerText = errorText
  classListAdd(input, BORDER_RED);
  classListRemove(input, FOCUS_RING)
  setVisible(false, repositoryInfo, checkMark, hint)

  disable(submitButton)
  swapElements(repositoryInfo, invalidFeedback)

  buttonText.innerText = submitText
  checkSpinner.hidden = true
}

function notValidUrlText(urlStr) {
  return '"' + urlStr + '" is not valid URL.'
}
function updateSelect(all_branches, preselected_branch) {
  dropdownList.innerHTML = ''
  const branches = all_branches.branches;
  const defaultBranch = all_branches.default_branch;

  for (var i = 0; i < branches.length; ++i) {
    const branchName = branches[i].name;
    const isDefaultBranch = branchName === defaultBranch

    const isPreselected = preselected_branch === branchName;
    const shouldSetCommitHash = preselected_branch === undefined ? isDefaultBranch : isPreselected;

    if (shouldSetCommitHash) {
      setCommitHash(branches[i].commit);
    }

    const dropdownItem = createListItem(
      branchName,
      shouldSetCommitHash,
      isDefaultBranch,
      branches[i].commit
    );

    branchLabel.innerText = preselected_branch ? preselected_branch : defaultBranch
    appendChildren(dropdownList, dropdownItem)
  }
}

function createListItem(branchName, isCurrentBranch, isDefaultBranch, commitHash) {
  const listItem = documentCreateElement("li")
  classListAdd(listItem, FLEX, CURSOR_POINTER, ITEMS_CENTER, SPACE_X_3, PY_2, HOVER_TEXT_BLACK, HOVER_ROUNDED, DARK_HOVER_ROUNDED_LG, DARK_HOVER_TEXT_NEUTRAL_100, DARK_BORDER_ZINC_500, DARK_TEXT_NEUTRAL_300)
  const mark = documentCreateElement(DIV)
  const text = documentCreateElement(TEXT)
  classListAdd(mark, W_2, PX_1, DARK_TEXT_NEUTRAL_300)
  text.innerText = branchName
  appendChildren(listItem, mark, text)
  if (isCurrentBranch) {
    mark.innerText = "✓"
  }
  if (isDefaultBranch) {
    const span = documentCreateElement("span")
    classListAdd(span, ROUNDED, BG_NEUTRAL_100, PX_2, TEXT_NEUTRAL_600, RING_1, RING_INSET, RING_GRAY_500_10, DARK_BORDER_ZINC_500, DARK_BG_ZINC_900, DARK_TEXT_NEUTRAL_300),
      span.innerText = DEFAULT
    appendChildren(listItem, span)
  }
  listItem.onclick = () => {
    updateSelect(Branches, branchName, commitHash)
  }
  return listItem
}

function updateRepositoryPicture() {
  const pic = documentGetElementById("repoButton")
  setVisible(true, pic)
  pic.innerHTML = ""
  pic.setAttribute("title", "Open repository " + input.value);

  repoUrl.setAttribute("href", input.value)
  const text = documentCreateElement(TEXT)
  classListAdd(text, HIDDEN, MD_BLOCK, PX_2)
  text.innerText = "Open " + extractRepositoryHost(input.value)

  appendChildren(pic, createRepositoryIcon(input.value, 24, 24), text)
}

function updateCommitLabel(branchName) {
  const branches_array = Branches.branches;
  for (let i = 0; i < branches_array.length; ++i) {
    if (branches_array[i].name === branchName) {
      setCommitHash(branches_array[i].commit)
    }
  }
}

function setCommitHash(commitHash) {
  classListAdd(commit, FLEX, TRUNCATE, ITEMS_CENTER, BORDER, BORDER_NEUTRAL, ROUNDED, DARK_BORDER_ZINC_500, DARK_BG_ZINC_800)
  const label = documentCreateElement(DIV)
  classListAdd(label, FLEX, "justify-center", ITEMS_CENTER, TEXT_SM, ROUNDED_L_LG, PY_2, PX_2, SM_PL_2, SM_PR_4, TEXT_NEUTRAL_600, BORDER_R, DARK_TEXT_NEUTRAL_200, DARK_BORDER_ZINC_500, DARK_BG_ZINC_800);
  const commitIcon = createCommitSvgIcon(20, 20)

  const text = documentCreateElement(TEXT)
  classListAdd(text, HIDDEN, SM_BLOCK)
  text.innerText = COMMIT
  appendChildren(label, commitIcon, text)

  const p = documentCreateElement("p")
  classListAdd(p, TRUNCATE, PX_2, SM_PR_4, FONT_MONO, FONT_LIGHT, TEXT_SM, TEXT_NEUTRAL_600, DARK_TEXT_NEUTRAL_300)
  p.innerText = commitHash
  commit.innerHTML = ""
  commit.title = LAST_COMMIT_SHA
  appendChildren(commit, label, p)
}

DOCUMENT.addEventListener('click', () => {
  Visibility -= 1
  if (Visibility < 0) {
    setVisible(false, dropdownList)
  }
});
