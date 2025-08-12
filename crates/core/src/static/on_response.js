function onHtmsResponse(id, html) {
    document.querySelector(`script[data-htms="${id}"]`).remove();
    document.querySelector(`[data-htms="${id}"]`).outerHTML = html;
}
