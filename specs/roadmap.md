# Roadmap

## Browser rendering through Chromium

Reintroduce `render` when it is needed by controlling an installed Chrome or
Chromium browser through `chromiumoxide` and the Chrome DevTools Protocol. Do
not bundle or download a browser as part of `pginf`. Design explicit process
lifecycle, hard timeouts, and browser-crash handling before implementation.
