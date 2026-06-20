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

function observeAboutSection() {
    const aboutSection = document.getElementById(ABOUT_SECTION_ID)

    if (!aboutSection) {
        return
    }

    if (!("IntersectionObserver" in globalThis)) {
        void loadAboutSection()
        return
    }

    const observer = new IntersectionObserver(
        (entries) => {
            for (let index = 0; index < entries.length; index += 1) {
                if (!entries[index].isIntersecting) {
                    continue
                }

                observer.disconnect()
                void loadAboutSection()
                break
            }
        },
        {
            rootMargin: "0px 0px 0px 0px"
        },
    )

    observer.observe(aboutSection)
}

function observeAfterScroll(callback) {
    let observed = false

    const start = () => {
        if (observed) {
            return
        }

        observed = true
        window.removeEventListener("scroll", start)
        window.removeEventListener("wheel", start)
        window.removeEventListener("touchmove", start)
        callback()
    }

    window.addEventListener("scroll", start, { passive: true, once: true })
    window.addEventListener("wheel", start, { passive: true, once: true })
    window.addEventListener("touchmove", start, { passive: true, once: true })
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => observeAfterScroll(observeAboutSection), { once: true })
} else {
    observeAfterScroll(observeAboutSection)
}