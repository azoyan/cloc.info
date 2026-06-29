const ABOUT_SECTION_ID = "aboutSection"
const ABOUT_SECTION_URL = "/assets/home-about.html"

async function loadAboutSection() {
    const aboutSection = document.getElementById(ABOUT_SECTION_ID)

    if (!aboutSection || aboutSection.dataset.loaded === "true" || aboutSection.dataset.loaded === "pending") {
        return
    }

    aboutSection.dataset.loaded = "pending"

    try {
        const response = await fetch(ABOUT_SECTION_URL)

        if (!response.ok) {
            throw new Error(`Request failed with status ${response.status}`)
        }

        aboutSection.innerHTML = await response.text()
        aboutSection.dataset.loaded = "true"
    } catch (error) {
        aboutSection.dataset.loaded = "error"
        console.error(error)
    }
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
        void loadAboutSection()
    }, { once: true })
} else {
    void loadAboutSection()
}