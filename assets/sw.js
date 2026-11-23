// StarBooru's service worker, registered by src/install.rs. It lets browsers
// install the app and lets the app start without a network. It keeps the
// app's own files and nothing else: what the server sends from the API and
// /media carries private data, and goes to the network untouched.
//
// - A page (any navigation): from the network, so a new version is picked up
//   as soon as there is one; the last page kept when the network fails.
// - A file of the bundle (/<name>.js, .wasm, .css, .png, .svg, .webmanifest):
//   from the cache, else from the network. Trunk names the scripts, the wasm
//   and the stylesheets after their content, so a kept one never goes stale;
//   the icons and the manifest keep their names, so a new page fetches them
//   again.
// - Anything else (the server's paths, other origins, anything but GET): not
//   answered here at all.
//
// The cache holds the page and the files it names. Each new page from the
// network takes the old one's place, the files it no longer names go, and the
// ones it names are fetched.

// A new name when the worker starts keeping things another way: activating
// deletes the app's other caches.
const CACHE = "starbooru-shell-1";
// Every address of the app is the same page, kept under this one.
const PAGE = "/";
const SERVER = /^\/(api|media|remote|backups|healthz)(\/|$)/;
const BUNDLE_FILE = /^\/[\w.-]+\.(js|wasm|css|png|svg|webmanifest)$/;
const NAMED = /["'](\/[\w.-]+\.(?:js|wasm|css|png|svg|webmanifest))["']/g;
// Trunk's names: `base-0123456789abcdef.css`, `starbooru-web-…_bg.wasm`.
const HASHED = /-[0-9a-f]{16}(_bg)?\.\w+$/;

self.addEventListener("install", (event) => {
  // Offline now, the files are kept as the app asks for them instead.
  event.waitUntil(refresh().catch(() => {}).then(() => self.skipWaiting()));
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((names) =>
      Promise.all(
        names
          .filter((name) => name.startsWith("starbooru-") && name !== CACHE)
          .map((name) => caches.delete(name)),
      ),
    ),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin || SERVER.test(url.pathname)) return;
  if (request.mode === "navigate") {
    event.respondWith(page(event));
  } else if (BUNDLE_FILE.test(url.pathname) && !url.search) {
    event.respondWith(bundleFile(event, url.pathname));
  }
});

async function page(event) {
  let response;
  try {
    response = await fetch(event.request);
  } catch {
    return (await caches.match(PAGE, { cacheName: CACHE })) ?? Response.error();
  }
  if (isPage(response)) {
    event.waitUntil(keepPage(response.clone()));
  } else if (response.status >= 500) {
    // A proxy whose server is down: the app starts, and says so.
    const kept = await caches.match(PAGE, { cacheName: CACHE });
    if (kept) return kept;
  }
  return response;
}

async function bundleFile(event, path) {
  const cache = await caches.open(CACHE);
  const hit = await cache.match(path, { ignoreVary: true });
  if (hit) return hit;
  const [response, names] = await Promise.all([fetch(event.request), named(cache)]);
  if (names.has(path) && isFile(response)) {
    event.waitUntil(cache.put(path, response.clone()));
  }
  return response;
}

// The first page, and the files it names, as soon as the worker installs.
async function refresh() {
  const response = await fetch(PAGE, { cache: "no-cache" });
  if (isPage(response)) await keepPage(response);
}

async function keepPage(response) {
  const cache = await caches.open(CACHE);
  const html = await response.clone().text();
  const before = await cache.match(PAGE);
  if (before && (await before.text()) === html) return;
  await cache.put(PAGE, response);
  const names = namesIn(html);
  for (const request of await cache.keys()) {
    const path = new URL(request.url).pathname;
    if (path !== PAGE && !names.has(path)) await cache.delete(request);
  }
  await Promise.all(
    [...names].map(async (name) => {
      if (HASHED.test(name) && (await cache.match(name, { ignoreVary: true }))) return;
      try {
        const file = await fetch(name, { cache: "no-cache" });
        if (isFile(file)) await cache.put(name, file);
      } catch {
        // Kept later, when the app asks for it.
      }
    }),
  );
}

async function named(cache) {
  const kept = await cache.match(PAGE);
  return kept ? namesIn(await kept.text()) : new Set();
}

function namesIn(html) {
  return new Set([...html.matchAll(NAMED)].map((match) => match[1]));
}

function isPage(response) {
  return response.status === 200 && !response.redirected && contentType(response).startsWith("text/html");
}

// The server answers a file it doesn't have with the page.
function isFile(response) {
  return response.status === 200 && !contentType(response).startsWith("text/html");
}

function contentType(response) {
  return response.headers.get("content-type") ?? "";
}
