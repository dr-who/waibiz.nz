# Waikato Entrepreneur / Biz Meetup

The single-page Leptos/WASM site for Waikato Entrepreneur / Biz Meetup, a gathering for current
and future entrepreneurs in Kirikiriroa and across Waikato.

Presented by:

- Hiko Hub (University of Waikato)
- Soda (Wintec)
- Altered Capital

## Development

```bash
env NO_COLOR=true trunk serve
```

The local site is served at `http://127.0.0.1:8080`.

## Production

```bash
env NO_COLOR=true trunk build --release
```

The deployable output is written to `dist/`. JavaScript and WASM filenames are
content-hashed by Trunk. Photographs below the fold are lazy-loaded and the
largest PNG sources have optimized WebP derivatives.

## Verification

```bash
npm install
npm run test:e2e -- --workers=1
```

The Playwright suite travels the full page at desktop and mobile sizes, checks
image loading, horizontal overflow, interactive state, console errors, WASM
loading and nonblank canvas pixels.

## Geometry And Assets

The river line is derived from OpenStreetMap relation `2751038`. The Lake Taupo
shoreline is derived from OpenStreetMap relation `1130806`. Both are attributed
to OpenStreetMap contributors under ODbL 1.0.

Partner marks and event photography are sourced from the official Hiko Hub,
University of Waikato, Soda, Wintec and Altered Capital sites. The Hiko Hub
photograph of Exaba executives and interns is from University of Waikato news.
The Port Waikato image is from Te Ara.

Every photograph and logo on the page is a link: partner images go to the
partner's site, event images go to the Eventbrite listing.
