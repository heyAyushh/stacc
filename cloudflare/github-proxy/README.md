# Cloudflare Worker: GitHub Proxy

This worker provides a **safe GitHub-only proxy** (not an open proxy) so your CLI can fetch files from GitHub via your own domain.

## Routes

- **`/raw/*`** → `raw.githubusercontent.com`
  - Example: `/raw/<owner>/<repo>/<ref>/<path>`
- **`/github/*`** → `github.com`
  - Example: `/github/<owner>/<repo>/releases/download/...`
- **`/api/*`** → `api.github.com`
  - Example: `/api/repos/<owner>/<repo>`

## Examples

Assuming your worker is hosted at `https://proxy.example.com`:

- Fetch `install.sh` from this repo via your domain:
  - `https://proxy.example.com/raw/heyAyushh/stacc/main/install.sh`
- GitHub API:
  - `https://proxy.example.com/api/repos/heyAyushh/stacc`

## Deploy

From this directory:

```bash
wrangler deploy
```

### Optional: increase GitHub rate limits

Set a GitHub token as a Worker secret (recommended):

```bash
wrangler secret put GITHUB_TOKEN
```

The worker will send `Authorization: Bearer <token>` upstream **only if** the client request did not already provide an `Authorization` header.

## Notes

- Supports **GET/HEAD/OPTIONS** only.
- Adds permissive **CORS** headers (`Access-Control-Allow-Origin: *`) to make browser-based tooling work too.
- Caches `/raw` and `/github` responses; `/api` responses are `no-store`.

