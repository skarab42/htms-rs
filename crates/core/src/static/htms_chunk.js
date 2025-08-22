class HTMSChunk extends HTMLElement {
    connectedCallback() {
        const target = this.getAttribute('target');
        const targetElement = document.querySelector(`[data-htms="${target}"]`);

        if (!targetElement) {
            console.warn('[htms-chunk] target not found:', target)
            return;
        }

        requestAnimationFrame(() => {
            targetElement.outerHTML = this.innerHTML;
            this.remove();
        });
    }
}

customElements.define('htms-chunk', HTMSChunk);

function htmsCleanup() {
    for (const element of document.querySelectorAll('.htms-dirty')) {
        element.remove();
    }
}
