/**
 * GitHub Proxy Worker (Cloudflare)
 *
 * Routes:
 *  - /raw/*    -> https://raw.githubusercontent.com/*
 *  - /github/* -> https://github.com/*
 *  - /api/*    -> https://api.github.com/*
 *
 * Security:
 *  - Not an open proxy: only these upstream hosts are allowed.
 *  - Only GET/HEAD/OPTIONS supported (safest for CLI use).
 *
 * Optional:
 *  - Bind a secret `GITHUB_TOKEN` to increase GitHub rate limits. If present, it will
 *    be sent as `Authorization: Bearer <token>` unless the client already provided Authorization.
 */

export interface Env {
  GITHUB_TOKEN?: string;
}

type Upstream = {
  host: "raw.githubusercontent.com" | "github.com" | "api.github.com";
  cacheMode: "cache" | "no-cache";
};

const ROUTES: Array<{ prefix: string; upstream: Upstream }> = [
  { prefix: "/raw/", upstream: { host: "raw.githubusercontent.com", cacheMode: "cache" } },
  { prefix: "/github/", upstream: { host: "github.com", cacheMode: "cache" } },
  { prefix: "/api/", upstream: { host: "api.github.com", cacheMode: "no-cache" } },
];

const CORS_HEADERS: Record<string, string> = {
  "access-control-allow-origin": "*",
  "access-control-allow-methods": "GET, HEAD, OPTIONS",
  "access-control-allow-headers": "Authorization, Content-Type, Accept, Range, User-Agent",
  "access-control-max-age": "86400",
};

function withCorsHeaders(headers: Headers) {
  for (const [k, v] of Object.entries(CORS_HEADERS)) headers.set(k, v);
}

function badRequest(msg: string, status = 400) {
  return new Response(msg + "\n", {
    status,
    headers: {
      "content-type": "text/plain; charset=utf-8",
      ...CORS_HEADERS,
    },
  });
}

function isSafePath(path: string): boolean {
  // Basic traversal / odd character protection.
  // We treat both decoded and raw forms defensively.
  if (path.includes("..")) return false;
  if (path.includes("\\")) return false;
  // Block encoded backslash (%5c) and encoded dot (%2e) sequences as well.
  const lower = path.toLowerCase();
  if (lower.includes("%5c")) return false;
  if (lower.includes("%2e%2e")) return false;
  return true;
}

function hopByHopHeader(name: string): boolean {
  // RFC 2616 hop-by-hop headers
  switch (name.toLowerCase()) {
    case "connection":
    case "keep-alive":
    case "proxy-authenticate":
    case "proxy-authorization":
    case "te":
    case "trailers":
    case "transfer-encoding":
    case "upgrade":
      return true;
    default:
      return false;
  }
}

function makeHelpPage(origin: string): Response {
  const body = [
    "GitHub Proxy Worker",
    "",
    "Routes:",
    `  ${origin}/raw/<owner>/<repo>/<ref>/<path>`,
    `  ${origin}/github/<owner>/<repo>/...`,
    `  ${origin}/api/<path>`,
    "",
    "Examples:",
    `  ${origin}/raw/heyAyushh/stacc/main/install.sh`,
    `  ${origin}/api/repos/heyAyushh/stacc`,
    "",
  ].join("\n");

  return new Response(body, {
    status: 200,
    headers: {
      "content-type": "text/plain; charset=utf-8",
      ...CORS_HEADERS,
      "cache-control": "public, max-age=300",
    },
  });
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: { ...CORS_HEADERS } });
    }

    if (request.method !== "GET" && request.method !== "HEAD") {
      return badRequest("Method not allowed", 405);
    }

    // Find matching route
    const match = ROUTES.find((r) => url.pathname === r.prefix.slice(0, -1) || url.pathname.startsWith(r.prefix));
    if (!match) {
      if (url.pathname === "/" || url.pathname === "") return makeHelpPage(url.origin);
      return badRequest("Unknown route. Use /raw/, /github/, or /api/.", 404);
    }

    const prefix = match.prefix;
    const upstream = match.upstream;

    // Compute upstream path, ensuring it begins with /
    const upstreamPath = url.pathname.startsWith(prefix) ? url.pathname.slice(prefix.length - 1) : "/";
    if (!isSafePath(upstreamPath)) return badRequest("Invalid path", 400);

    const upstreamUrl = new URL(`https://${upstream.host}${upstreamPath}${url.search}`);

    // Prepare upstream request headers
    const upstreamHeaders = new Headers();
    for (const [k, v] of request.headers.entries()) {
      if (hopByHopHeader(k)) continue;
      if (k.toLowerCase() === "host") continue;
      // Cloudflare & forwarding headers are not useful upstream.
      if (k.toLowerCase().startsWith("cf-")) continue;
      if (k.toLowerCase().startsWith("x-forwarded-")) continue;
      upstreamHeaders.set(k, v);
    }

    // Ensure GitHub has a UA (some endpoints behave better with it).
    if (!upstreamHeaders.has("user-agent")) {
      upstreamHeaders.set("user-agent", "stacc-github-proxy/1.0");
    }

    // Optional token injection for rate limits (do not override client-provided auth).
    if (!upstreamHeaders.has("authorization") && env.GITHUB_TOKEN && env.GITHUB_TOKEN.trim().length > 0) {
      upstreamHeaders.set("authorization", `Bearer ${env.GITHUB_TOKEN.trim()}`);
    }

    // Conservative caching:
    // - /api: no caching (responses vary and may include auth-dependent data)
    // - /raw and /github: cache successful responses for speed
    const cf =
      upstream.cacheMode === "cache"
        ? {
            cacheEverything: true,
            cacheTtlByStatus: {
              "200-299": 3600,
              "300-399": 300,
              "404": 60,
              "500-599": 0,
            },
          }
        : { cacheTtl: 0 };

    const upstreamReq = new Request(upstreamUrl.toString(), {
      method: request.method,
      headers: upstreamHeaders,
      redirect: "follow",
      // No body for GET/HEAD.
      cf,
    } as RequestInit);

    let res = await fetch(upstreamReq);

    // Make response mutable so we can append CORS + avoid leaking upstream security headers.
    const resHeaders = new Headers(res.headers);
    withCorsHeaders(resHeaders);

    // Avoid caching by intermediaries if upstream is api
    if (upstream.cacheMode === "no-cache") {
      resHeaders.set("cache-control", "no-store");
    }

    // Helpful debugging: expose upstream target (non-sensitive).
    resHeaders.set("x-upstream-host", upstream.host);

    return new Response(res.body, {
      status: res.status,
      statusText: res.statusText,
      headers: resHeaders,
    });
  },
};

