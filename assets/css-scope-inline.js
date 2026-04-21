// 🌘 CSS Scope Inline (https://github.com/gnat/css-scope-inline)
window.cssScopeCount ??= 1 // Let extra copies share the scope count.
window.cssScopeByStableId ??= new Map()
window.cssStyleByStableId ??= new Map()

function applyScopeTransforms(cssText, scope) {
    return cssText
        .replace(/(?:^|\.|(\s|[^a-zA-Z0-9\-\_]))(me|this|self)(?![a-zA-Z])/g, '$1[data-mx-css-scope="' + scope + '"]') // Can use: me this self
        .replace(/((@keyframes|animation:|animation-name:)[^{};]*)\.me__/g, '$1me__') // Optional. Removes need to escape names, ex: "\.me"
        .replace(/(?:@media)\s(xs-|sm-|md-|lg-|xl-|sm|md|lg|xl|xx)/g, // Optional. Responsive design. Mobile First (above breakpoint): 🟢 None sm md lg xl xx 🏁  Desktop First (below breakpoint): 🏁 xs- sm- md- lg- xl- None 🟢 *- matches must be first!
            (match, part1) => { return '@media ' + ({ 'sm': '(min-width: 640px)', 'md': '(min-width: 768px)', 'lg': '(min-width: 1024px)', 'xl': '(min-width: 1280px)', 'xx': '(min-width: 1536px)', 'xs-': '(max-width: 639px)', 'sm-': '(max-width: 767px)', 'md-': '(max-width: 1023px)', 'lg-': '(max-width: 1279px)', 'xl-': '(max-width: 1535px)' }[part1]) }
        )
}

function scopeClassFor(stableId) {
    if (!stableId) {
        return 'me__' + (window.cssScopeCount++)
    }
    window.cssScopeByStableId.set(stableId, stableId)
    return stableId
}

function scopePendingStyles() {
    document?.querySelectorAll('style[data-mx-css-id]:not([ready])').forEach(node => { // Faster than walking MutationObserver results when recieving subtree (DOM swap, ajax, jquery).
        const stableId = node.getAttribute('data-mx-css-id')
        const scope = scopeClassFor(stableId)
        if (node.parentNode && !node.parentNode.getAttribute('data-mx-css-scope')) {
            node.parentNode.setAttribute('data-mx-css-scope', scope)
        }

        if (stableId) {
            const canonical = window.cssStyleByStableId.get(stableId)
            if (canonical) {
                node.setAttribute('ready', '')
                node.remove()
                return
            }
        }

        node.textContent = applyScopeTransforms(node.textContent || '', scope)
        node.setAttribute('ready', '')

        if (stableId) {
            window.cssStyleByStableId.set(stableId, node)
            if (document.head && node.parentNode !== document.head) {
                document.head.appendChild(node)
            }
        }
    })
}

window.cssScope ??= new MutationObserver(() => { // Allow 1 observer.
    scopePendingStyles()
})

scopePendingStyles()
window.cssScope.observe(document.documentElement, { childList: true, subtree: true })
