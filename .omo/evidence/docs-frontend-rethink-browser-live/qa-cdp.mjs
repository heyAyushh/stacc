const BASE_URL = 'http://127.0.0.1:4174';
const CDP_PORT = process.env.CDP_PORT || '9224';
const ARTIFACT_DIR = '.omo/evidence/docs-frontend-rethink-browser-live';
const ROUTES = ['/', '/docs/getting-started', '/docs/binary-tui', '/docs/skills', '/docs/architecture'];
const VIEWPORTS = [
  { name: 'desktop', width: 1280, height: 900, deviceScaleFactor: 1, mobile: false },
  { name: 'mobile', width: 390, height: 844, deviceScaleFactor: 2, mobile: true },
];

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const slug = (route) => (route === '/' ? 'root' : route.replace(/^\//, '').replaceAll('/', '-'));

async function httpJson(url, options) {
  const response = await fetch(url, options);
  if (!response.ok) throw new Error(`${options?.method || 'GET'} ${url} -> ${response.status}`);
  return response.json();
}

class CdpClient {
  constructor(wsUrl) {
    this.nextId = 1;
    this.pending = new Map();
    this.handlers = new Map();
    this.ws = new WebSocket(wsUrl);
  }
  async open() {
    await new Promise((resolve, reject) => {
      this.ws.addEventListener('open', resolve, { once: true });
      this.ws.addEventListener('error', reject, { once: true });
    });
    this.ws.addEventListener('message', (event) => {
      const message = JSON.parse(event.data);
      if (message.id && this.pending.has(message.id)) {
        const { resolve, reject } = this.pending.get(message.id);
        this.pending.delete(message.id);
        if (message.error) reject(new Error(`${message.error.message}: ${message.error.data || ''}`));
        else resolve(message.result || {});
        return;
      }
      if (message.method && this.handlers.has(message.method)) {
        for (const handler of this.handlers.get(message.method)) handler(message.params || {});
      }
    });
  }
  on(method, handler) {
    if (!this.handlers.has(method)) this.handlers.set(method, []);
    this.handlers.get(method).push(handler);
  }
  send(method, params = {}) {
    const id = this.nextId++;
    this.ws.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error(`CDP timeout: ${method}`));
        }
      }, 15000);
    });
  }
  close() {
    this.ws.close();
  }
}

async function waitFor(client, method, trigger, timeoutMs = 20000) {
  return new Promise(async (resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`Timed out waiting for ${method}`)), timeoutMs);
    client.on(method, (params) => {
      clearTimeout(timer);
      resolve(params);
    });
    try {
      await trigger();
    } catch (error) {
      clearTimeout(timer);
      reject(error);
    }
  });
}

async function evaluate(client, expression, returnByValue = true) {
  const result = await client.send('Runtime.evaluate', {
    expression,
    awaitPromise: true,
    returnByValue,
    userGesture: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text || 'Runtime.evaluate exception');
  }
  return result.result?.value;
}

async function clickSelector(client, selector) {
  const box = await evaluate(client, `(() => {
    const el = document.querySelector(${JSON.stringify(selector)});
    if (!el) return null;
    const rect = el.getBoundingClientRect();
    return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2, width: rect.width, height: rect.height };
  })()`);
  if (!box) throw new Error(`Missing selector ${selector}`);
  await client.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: box.x, y: box.y });
  await client.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: box.x, y: box.y, button: 'left', clickCount: 1 });
  await client.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: box.x, y: box.y, button: 'left', clickCount: 1 });
  return box;
}

async function waitForExpression(client, expression, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const value = await evaluate(client, expression);
    if (value) return value;
    await sleep(100);
  }
  throw new Error(`Timed out waiting for expression: ${expression}`);
}

const target = await httpJson(`http://127.0.0.1:${CDP_PORT}/json/new?about:blank`, { method: 'PUT' });
const client = new CdpClient(target.webSocketDebuggerUrl);
await client.open();

const browserErrors = [];
client.on('Runtime.consoleAPICalled', (params) => {
  if (params.type === 'error') browserErrors.push({ kind: 'console', text: params.args?.map((arg) => arg.value || arg.description).join(' ') });
});
client.on('Runtime.exceptionThrown', (params) => {
  browserErrors.push({ kind: 'pageerror', text: params.exceptionDetails?.text || params.exceptionDetails?.exception?.description || 'exception' });
});
client.on('Log.entryAdded', (params) => {
  if (params.entry?.level === 'error') browserErrors.push({ kind: 'log', text: params.entry.text });
});

await client.send('Page.enable');
await client.send('Runtime.enable');
await client.send('Log.enable');
await client.send('DOM.enable');
try {
  await client.send('Browser.grantPermissions', {
    origin: BASE_URL,
    permissions: ['clipboardReadWrite', 'clipboardSanitizedWrite'],
  });
} catch (error) {
  browserErrors.push({ kind: 'permission-warning', text: error.message });
}

const observations = [];
const failures = [];

for (const viewport of VIEWPORTS) {
  await client.send('Emulation.setDeviceMetricsOverride', viewport);
  for (const route of ROUTES) {
    const beforeErrorCount = browserErrors.length;
    const url = `${BASE_URL}${route}`;
    await waitFor(client, 'Page.loadEventFired', () => client.send('Page.navigate', { url }));
    await waitForExpression(client, 'document.readyState === "complete"');
    await sleep(500);

    const metrics = await evaluate(client, `(() => {
      const doc = document.documentElement;
      const editable = Array.from(document.querySelectorAll('input, textarea, select, [contenteditable="true"], [role="textbox"]'))
        .map((el) => ({ tag: el.tagName, role: el.getAttribute('role'), aria: el.getAttribute('aria-label'), fontSize: parseFloat(getComputedStyle(el).fontSize), selector: el.className || el.id || el.tagName }))
        .filter((item) => item.fontSize < 16);
      const overflowing = Array.from(document.body.querySelectorAll('*')).map((el) => {
        const rect = el.getBoundingClientRect();
        const style = getComputedStyle(el);
        return { tag: el.tagName, className: String(el.className || ''), left: rect.left, right: rect.right, width: rect.width, overflowX: style.overflowX, position: style.position };
      }).filter((item) => item.width > 0 && item.left < -1 || item.right > window.innerWidth + 1).slice(0, 20);
      return {
        title: document.title,
        scrollWidth: doc.scrollWidth,
        clientWidth: doc.clientWidth,
        bodyScrollWidth: document.body.scrollWidth,
        viewportWidth: window.innerWidth,
        hasHorizontalOverflow: doc.scrollWidth > doc.clientWidth,
        smallEditableFonts: editable,
        overflowingElements: overflowing,
      };
    })()`);

    const screenshotPath = `${ARTIFACT_DIR}/${viewport.name}-${slug(route)}.png`;
    const screenshot = await client.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
    await BunLikeWriteFile(screenshotPath, Buffer.from(screenshot.data, 'base64'));

    const routeErrors = browserErrors.slice(beforeErrorCount).filter((error) => error.kind !== 'permission-warning');
    const observation = { route, viewport: viewport.name, url, metrics, routeErrors, screenshotPath };
    observations.push(observation);
    if (!metrics.title) failures.push(`${viewport.name} ${route}: empty title`);
    if (routeErrors.length) failures.push(`${viewport.name} ${route}: browser errors ${JSON.stringify(routeErrors)}`);
    if (metrics.hasHorizontalOverflow) failures.push(`${viewport.name} ${route}: document overflow ${metrics.scrollWidth} > ${metrics.clientWidth}`);
    if (metrics.smallEditableFonts.length) failures.push(`${viewport.name} ${route}: small editable fonts ${JSON.stringify(metrics.smallEditableFonts)}`);
  }
}

await client.send('Emulation.setDeviceMetricsOverride', VIEWPORTS[0]);
await waitFor(client, 'Page.loadEventFired', () => client.send('Page.navigate', { url: `${BASE_URL}/` }));
await waitForExpression(client, 'document.readyState === "complete"');
await sleep(500);
await clickSelector(client, '.install-card');
const landingCopy = await waitForExpression(client, `document.querySelector('.install-card')?.dataset.copyState === 'copied' && document.querySelector('.install-card')?.dataset.copyState`, 5000);
const landingShot = `${ARTIFACT_DIR}/interaction-landing-copy.png`;
await BunLikeWriteFile(landingShot, Buffer.from((await client.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false })).data, 'base64'));

await waitFor(client, 'Page.loadEventFired', () => client.send('Page.navigate', { url: `${BASE_URL}/docs/getting-started` }));
await waitForExpression(client, 'document.readyState === "complete"');
await sleep(500);
await clickSelector(client, 'button.copy-panel');
const docsCopy = await waitForExpression(client, `document.querySelector('button.copy-panel')?.dataset.copyState === 'copied' && document.querySelector('button.copy-panel')?.dataset.copyState`, 5000);
const docsCopyShot = `${ARTIFACT_DIR}/interaction-docs-code-copy.png`;
await BunLikeWriteFile(docsCopyShot, Buffer.from((await client.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false })).data, 'base64'));

await clickSelector(client, '.search-field');
await sleep(300);
const searchState = await evaluate(client, `(() => {
  const dialog = document.querySelector('[role="dialog"], .command-dialog, .search-dialog');
  const input = document.querySelector('[role="dialog"] input, .command-dialog input, .search-dialog input, input[type="search"], input');
  return {
    hasDialog: Boolean(dialog),
    activeTag: document.activeElement?.tagName,
    activeRole: document.activeElement?.getAttribute('role'),
    inputFontSize: input ? parseFloat(getComputedStyle(input).fontSize) : null,
    inputPlaceholder: input ? input.getAttribute('placeholder') : null,
    bodyTextSample: document.body.innerText.slice(0, 500),
  };
})()`);
const searchShot = `${ARTIFACT_DIR}/interaction-command-search.png`;
await BunLikeWriteFile(searchShot, Buffer.from((await client.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false })).data, 'base64'));
if (!searchState.hasDialog) failures.push(`command search did not open: ${JSON.stringify(searchState)}`);
if (searchState.inputFontSize !== null && searchState.inputFontSize < 16) failures.push(`command search input font ${searchState.inputFontSize} < 16`);

const summary = {
  generatedAt: new Date().toISOString(),
  surface: 'live docs app at http://127.0.0.1:4174/',
  browser: 'Google Chrome via Chrome DevTools Protocol',
  routes: ROUTES,
  viewports: VIEWPORTS.map(({ name, width, height }) => ({ name, width, height })),
  interactions: { landingCopy, docsCopy, searchState, landingShot, docsCopyShot, searchShot },
  observations,
  browserErrors,
  failures,
  verdict: failures.length === 0 ? 'PASS' : 'FAIL',
};
await BunLikeWriteFile(`${ARTIFACT_DIR}/browser-qa-results.json`, JSON.stringify(summary, null, 2));
client.close();
if (failures.length) {
  console.error(JSON.stringify(summary, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(summary, null, 2));

async function BunLikeWriteFile(path, data) {
  const fs = await import('node:fs/promises');
  await fs.writeFile(path, data);
}
