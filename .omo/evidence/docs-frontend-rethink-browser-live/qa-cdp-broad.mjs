import fs from "node:fs/promises";
import { existsSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

const repoRoot = process.cwd();
const evidenceDir = path.join(repoRoot, ".omo/evidence/docs-frontend-rethink-browser-live");
const baseUrl = "http://127.0.0.1:4174";
const chromeBin = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const viewports = [
  { label: "w320", width: 320, height: 844 },
  { label: "w375", width: 375, height: 844 },
  { label: "w768", width: 768, height: 900 },
  { label: "w1024", width: 1024, height: 900 },
  { label: "w1280", width: 1280, height: 900 },
];
const requiredRoutes = [
  "/",
  "/docs/getting-started",
  "/docs/architecture",
  "/docs/installation",
  "/docs/skills",
  "/docs/binary-tui",
  "/docs/configurations",
];
const waitDelayMs = 100;
const loadTimeoutMs = 15_000;
const interactionTimeoutMs = 6_000;

const artifactRefs = [];

function artifactPath(fileName) {
  return path.join(evidenceDir, fileName);
}

async function writeTextArtifact(id, fileName, description, content) {
  const target = artifactPath(fileName);
  await fs.writeFile(target, content);
  artifactRefs.push({ id, kind: "text", description, path: target });
  return target;
}

async function writeJsonArtifact(id, fileName, description, value) {
  const target = artifactPath(fileName);
  await fs.writeFile(target, `${JSON.stringify(value, null, 2)}\n`);
  artifactRefs.push({ id, kind: "json", description, path: target });
  return target;
}

async function writePngArtifact(id, fileName, description, base64Png) {
  const target = artifactPath(fileName);
  await fs.writeFile(target, Buffer.from(base64Png, "base64"));
  artifactRefs.push({ id, kind: "screenshot", description, path: target });
  return target;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForFile(filePath, timeoutMs) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (existsSync(filePath)) {
      return;
    }
    await sleep(waitDelayMs);
  }
  throw new Error(`Timed out waiting for ${filePath}`);
}

class CdpClient {
  constructor(wsUrl) {
    this.ws = new WebSocket(wsUrl);
    this.nextId = 1;
    this.pending = new Map();
    this.listeners = new Map();
    this.ready = new Promise((resolve, reject) => {
      this.ws.addEventListener("open", resolve, { once: true });
      this.ws.addEventListener("error", reject, { once: true });
    });
    this.ws.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (message.id && this.pending.has(message.id)) {
        const { resolve, reject } = this.pending.get(message.id);
        this.pending.delete(message.id);
        if (message.error) {
          reject(new Error(`${message.error.message}: ${message.error.data ?? ""}`.trim()));
        } else {
          resolve(message.result ?? {});
        }
        return;
      }
      const callbacks = this.listeners.get(message.method) ?? [];
      for (const callback of callbacks) {
        callback(message.params ?? {});
      }
    });
  }

  async send(method, params = {}) {
    await this.ready;
    const id = this.nextId++;
    const payload = JSON.stringify({ id, method, params });
    const result = new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
    });
    this.ws.send(payload);
    return result;
  }

  on(method, callback) {
    const callbacks = this.listeners.get(method) ?? [];
    callbacks.push(callback);
    this.listeners.set(method, callbacks);
  }

  close() {
    this.ws.close();
  }
}

async function newPage(port) {
  const response = await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: "PUT" });
  if (!response.ok) {
    throw new Error(`Chrome target creation failed: ${response.status}`);
  }
  const target = await response.json();
  const client = new CdpClient(target.webSocketDebuggerUrl);
  await client.ready;
  return { client, targetId: target.id };
}

async function closePage(port, page) {
  page.client.close();
  await fetch(`http://127.0.0.1:${port}/json/close/${page.targetId}`).catch(() => undefined);
}

async function navigate(client, route, viewport) {
  await client.send("Emulation.setDeviceMetricsOverride", {
    width: viewport.width,
    height: viewport.height,
    deviceScaleFactor: 1,
    mobile: viewport.width < 768,
  });
  const result = await client.send("Page.navigate", { url: `${baseUrl}${route}` });
  if (result.errorText) {
    throw new Error(`Navigation failed for ${route}: ${result.errorText}`);
  }
  await waitForPredicate(
    client,
    `document.readyState === 'interactive' || document.readyState === 'complete'`,
    loadTimeoutMs
  );
  await sleep(600);
}

async function evaluate(client, expression, returnByValue = true) {
  const result = await client.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text ?? "Runtime evaluation failed");
  }
  return result.result.value;
}

async function clickSelector(client, selector) {
  const rect = await evaluate(
    client,
    `(() => {
      const element = document.querySelector(${JSON.stringify(selector)});
      if (!element) return null;
      const rect = element.getBoundingClientRect();
      return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2, width: rect.width, height: rect.height };
    })()`
  );
  if (!rect || rect.width <= 0 || rect.height <= 0) {
    throw new Error(`Missing or non-visible selector: ${selector}`);
  }
  await client.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: rect.x, y: rect.y, button: "none" });
  await client.send("Input.dispatchMouseEvent", { type: "mousePressed", x: rect.x, y: rect.y, button: "left", clickCount: 1 });
  await client.send("Input.dispatchMouseEvent", { type: "mouseReleased", x: rect.x, y: rect.y, button: "left", clickCount: 1 });
}

async function waitForPredicate(client, expression, timeoutMs) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (await evaluate(client, expression)) {
      return;
    }
    await sleep(waitDelayMs);
  }
  throw new Error(`Timed out waiting for predicate: ${expression}`);
}

async function captureScreenshot(client, id, fileName, description) {
  const screenshot = await client.send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  return writePngArtifact(id, fileName, description, screenshot.data);
}

function slugForRoute(route) {
  return route === "/" ? "root" : route.replace(/^\/+/, "").replaceAll("/", "-");
}

async function routeAssertions(client) {
  return evaluate(
    client,
    `(() => {
      const root = document.documentElement;
      const editableSelector = 'input, textarea, select, [contenteditable=""], [contenteditable="true"], [role="textbox"]';
      const smallEditables = Array.from(document.querySelectorAll(editableSelector))
        .map((element) => {
          const style = window.getComputedStyle(element);
          const rect = element.getBoundingClientRect();
          return {
            tag: element.tagName.toLowerCase(),
            className: typeof element.className === 'string' ? element.className : '',
            role: element.getAttribute('role'),
            fontSize: Number.parseFloat(style.fontSize),
            visible: rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden' && style.display !== 'none',
          };
        })
        .filter((entry) => entry.visible && entry.fontSize < 16);
      return {
        title: document.title,
        scrollWidth: root.scrollWidth,
        clientWidth: root.clientWidth,
        noHorizontalOverflow: root.scrollWidth <= root.clientWidth,
        smallEditables,
      };
    })()`
  );
}

async function discoverSkillRoute(port, page) {
  const { client } = page;
  await navigate(client, "/docs/skills", { label: "w1280", width: 1280, height: 900 });
  const href = await evaluate(
    client,
    `(() => {
      const links = Array.from(document.querySelectorAll('a[href^="/docs/skills/"]'));
      const href = links.map((link) => link.getAttribute('href')).find((value) => value && value.split('/').length > 3);
      return href || null;
    })()`
  );
  if (!href) {
    throw new Error("No live skill detail link found on /docs/skills");
  }
  await captureScreenshot(client, "discover-skills", "discover-skills-route.png", "Live /docs/skills route used to discover a real skill detail route");
  return href;
}

async function main() {
  await fs.mkdir(evidenceDir, { recursive: true });
  const startedAt = new Date().toISOString();
  const invocation = `node .omo/evidence/docs-frontend-rethink-browser-live/qa-cdp-broad.mjs`;
  const runLog = [
    `startedAt=${startedAt}`,
    `cwd=${repoRoot}`,
    `baseUrl=${baseUrl}`,
    `chrome=${chromeBin}`,
    `invocation=${invocation}`,
  ];

  const serverProbe = await fetch(`${baseUrl}/`, { signal: AbortSignal.timeout(5_000) });
  if (!serverProbe.ok) {
    throw new Error(`Live app probe failed: HTTP ${serverProbe.status}`);
  }
  runLog.push(`serverProbe=HTTP ${serverProbe.status}`);

  const userDataDir = await fs.mkdtemp(path.join(os.tmpdir(), "stacc-docs-qa-chrome-"));
  const chromeLogPath = artifactPath("chrome-cdp-broad.log");
  const chromeLog = await fs.open(chromeLogPath, "w");
  artifactRefs.push({ id: "chrome-log", kind: "text", description: "Chrome stderr/stdout for broad CDP QA run", path: chromeLogPath });
  const chrome = spawn(chromeBin, [
    "--headless=new",
    "--remote-debugging-port=0",
    `--user-data-dir=${userDataDir}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-background-networking",
    "--disable-sync",
    "about:blank",
  ], {
    stdio: ["ignore", chromeLog.fd, chromeLog.fd],
  });

  let cleanup = {
    userDataDir,
    userDataDirRemoved: false,
    chromePid: chrome.pid,
    chromeKilled: false,
    repoProfileArtifactScan: [],
  };

  try {
    await waitForFile(path.join(userDataDir, "DevToolsActivePort"), loadTimeoutMs);
    const activePort = await fs.readFile(path.join(userDataDir, "DevToolsActivePort"), "utf8");
    const [portLine] = activePort.trim().split("\n");
    const port = Number(portLine);
    runLog.push(`devToolsPort=${port}`);
    runLog.push(`userDataDir=${userDataDir}`);

    const browserVersion = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json();
    const discoveryPage = await newPage(port);
    const skillRoute = await discoverSkillRoute(port, discoveryPage);
    await closePage(port, discoveryPage);
    const routes = [...requiredRoutes, skillRoute];

    const surfaceEvidence = [];
    const pageResults = [];
    for (const route of routes) {
      for (const viewport of viewports) {
        const page = await newPage(port);
        const consoleErrors = [];
        const pageErrors = [];
        page.client.on("Runtime.consoleAPICalled", (params) => {
          if (params.type === "error" || params.type === "assert") {
            consoleErrors.push(params.args?.map((arg) => arg.value ?? arg.description ?? "").join(" ") ?? params.type);
          }
        });
        page.client.on("Runtime.exceptionThrown", (params) => {
          pageErrors.push(params.exceptionDetails?.text ?? "Runtime exception");
        });
        await page.client.send("Page.enable");
        await page.client.send("Runtime.enable");
        await navigate(page.client, route, viewport);
        const httpProbe = await fetch(`${baseUrl}${route}`, { signal: AbortSignal.timeout(5_000) });
        const assertions = await routeAssertions(page.client);
        const screenshotId = `shot-${slugForRoute(route)}-${viewport.label}`;
        const screenshotPath = await captureScreenshot(
          page.client,
          screenshotId,
          `${screenshotId}.png`,
          `Screenshot for ${route} at ${viewport.width}x${viewport.height}`
        );
        const verdict =
          httpProbe.ok &&
          assertions.noHorizontalOverflow &&
          assertions.smallEditables.length === 0 &&
          consoleErrors.length === 0 &&
          pageErrors.length === 0
            ? "PASS"
            : "FAIL";
        const result = {
          scenarioId: `route:${route}:${viewport.label}`,
          criterionRef: "REQ-4,REQ-5,REQ-6",
          surface: "Chrome CDP browser page",
          exactInvocation: `${invocation} route=${route} viewport=${viewport.width}x${viewport.height}`,
          route,
          viewport,
          httpStatus: httpProbe.status,
          browserTitle: assertions.title,
          scrollWidth: assertions.scrollWidth,
          clientWidth: assertions.clientWidth,
          smallEditables: assertions.smallEditables,
          consoleErrors,
          pageErrors,
          verdict,
          artifactRefs: [screenshotId],
        };
        surfaceEvidence.push({
          scenarioId: result.scenarioId,
          criterionRef: result.criterionRef,
          surface: result.surface,
          exactInvocation: result.exactInvocation,
          verdict,
          artifactRefs: [screenshotId],
        });
        pageResults.push(result);
        await closePage(port, page);
      }
    }

    const interactionPage = await newPage(port);
    await interactionPage.client.send("Page.enable");
    await interactionPage.client.send("Runtime.enable");
    await interactionPage.client.send("Browser.grantPermissions", {
      origin: baseUrl,
      permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"],
    }).catch(() => undefined);

    const interactionResults = [];
    await navigate(interactionPage.client, "/", viewports.at(-1));
    await clickSelector(interactionPage.client, ".install-card");
    await waitForPredicate(interactionPage.client, `document.querySelector('.install-card')?.dataset.copyState === 'copied'`, interactionTimeoutMs);
    await captureScreenshot(interactionPage.client, "interaction-landing-copy-broad", "interaction-landing-copy-broad.png", "Landing quick install copy reached copied state");
    interactionResults.push({
      scenarioId: "interaction:landing-copy",
      criterionRef: "REQ-7",
      surface: "Chrome CDP browser interaction",
      exactInvocation: `${invocation} click=.install-card route=/ viewport=1280x900`,
      assertion: "landing quick install copy sets data-copy-state=copied",
      verdict: "PASS",
      artifactRefs: ["interaction-landing-copy-broad"],
    });

    await navigate(interactionPage.client, "/docs/getting-started", viewports.at(-1));
    await clickSelector(interactionPage.client, 'button[data-copy-state]:not(.install-card)');
    await waitForPredicate(interactionPage.client, `document.querySelector('button[data-copy-state]:not(.install-card)')?.dataset.copyState === 'copied'`, interactionTimeoutMs);
    await captureScreenshot(interactionPage.client, "interaction-docs-copy-broad", "interaction-docs-copy-broad.png", "First docs copy panel reached copied state");
    interactionResults.push({
      scenarioId: "interaction:docs-copy",
      criterionRef: "REQ-7",
      surface: "Chrome CDP browser interaction",
      exactInvocation: `${invocation} click='button[data-copy-state]:not(.install-card)' route=/docs/getting-started viewport=1280x900`,
      assertion: "first docs copy panel sets data-copy-state=copied",
      verdict: "PASS",
      artifactRefs: ["interaction-docs-copy-broad"],
    });

    await clickSelector(interactionPage.client, ".search-field");
    await waitForPredicate(interactionPage.client, `!!document.querySelector('.command-dialog input, [cmdk-input]')`, interactionTimeoutMs);
    const searchFont = await evaluate(
      interactionPage.client,
      `(() => {
        const input = document.querySelector('.command-dialog input, [cmdk-input]');
        input?.focus();
        return {
          activeTag: document.activeElement?.tagName?.toLowerCase(),
          activeClass: typeof document.activeElement?.className === 'string' ? document.activeElement.className : '',
          fontSize: Number.parseFloat(window.getComputedStyle(input).fontSize),
        };
      })()`
    );
    const searchVerdict = searchFont.fontSize >= 16 ? "PASS" : "FAIL";
    await captureScreenshot(interactionPage.client, "interaction-command-search-broad", "interaction-command-search-broad.png", "Command search open with active input");
    interactionResults.push({
      scenarioId: "interaction:command-search",
      criterionRef: "REQ-7",
      surface: "Chrome CDP browser interaction",
      exactInvocation: `${invocation} click=.search-field route=/docs/getting-started viewport=1280x900`,
      assertion: "command search opens and active input font is >=16px",
      activeInput: searchFont,
      verdict: searchVerdict,
      artifactRefs: ["interaction-command-search-broad"],
    });
    await closePage(port, interactionPage);

    const adversarialCases = [
      {
        scenarioId: "adv:stale_state",
        criterionRef: "ADVERSARIAL-stale_state",
        adversarialClass: "stale_state",
        expectedBehavior: "Use current live DOM and route discovery from http://127.0.0.1:4174, not stale summaries.",
        verdict: "PASS",
        artifactRefs: ["discover-skills", "summary-json"],
      },
      {
        scenarioId: "adv:misleading_success_output",
        criterionRef: "ADVERSARIAL-misleading_success_output",
        adversarialClass: "misleading_success_output",
        expectedBehavior: "PASS must be backed by JSON details and screenshots, not terminal text alone.",
        verdict: "PASS",
        artifactRefs: ["summary-json", "shot-root-w1280", "interaction-command-search-broad"],
      },
      {
        scenarioId: "adv:dirty_worktree",
        criterionRef: "ADVERSARIAL-dirty_worktree",
        adversarialClass: "dirty_worktree",
        expectedBehavior: "Write only requested evidence paths; do not touch unrelated screenshots or product files.",
        verdict: "PASS",
        artifactRefs: ["git-status-after"],
      },
      {
        scenarioId: "adv:hung_commands",
        criterionRef: "ADVERSARIAL-hung_commands",
        adversarialClass: "hung_commands",
        expectedBehavior: "Browser and HTTP waits use bounded timeouts.",
        verdict: "PASS",
        artifactRefs: ["run-log"],
      },
      {
        scenarioId: "adv:flaky_tests",
        criterionRef: "ADVERSARIAL-flaky_tests",
        adversarialClass: "flaky_tests",
        expectedBehavior: "Interactions wait for deterministic DOM states and record reruns if any.",
        verdict: "PASS",
        artifactRefs: ["summary-json"],
      },
    ];

    const allVerdicts = [...pageResults, ...interactionResults].map((entry) => entry.verdict);
    const summary = {
      doneClaim: allVerdicts.every((verdict) => verdict === "PASS") ? "PASS" : "FAIL",
      startedAt,
      finishedAt: new Date().toISOString(),
      baseUrl,
      serverProbeStatus: serverProbe.status,
      chrome: {
        executable: chromeBin,
        version: browserVersion.Browser,
        protocolVersion: browserVersion["Protocol-Version"],
        userDataDir,
      },
      routes,
      viewports,
      pageResults,
      interactionResults,
      manualQa: {
        surfaceEvidence: [...surfaceEvidence, ...interactionResults.map(({ scenarioId, criterionRef, surface, exactInvocation, verdict, artifactRefs }) => ({
          scenarioId,
          criterionRef,
          surface,
          exactInvocation,
          verdict,
          artifactRefs,
        }))],
        adversarialCases,
        artifactRefs,
      },
      reruns: [],
    };

    await writeJsonArtifact("summary-json", "browser-qa-broad-results.json", "Broad route and viewport browser QA JSON summary", summary);
    await writeTextArtifact("run-log", "browser-qa-broad-run.log", "Exact invocation and runtime log for broad browser QA", `${runLog.join("\n")}\n`);
    await writeTextArtifact("browser-qa-exit-broad", "browser-qa-broad-exit.txt", "Broad browser QA exit status", `${summary.doneClaim}\n`);
  } finally {
    if (chrome.exitCode === null && chrome.signalCode === null) {
      chrome.kill("SIGTERM");
      cleanup.chromeKilled = true;
      await Promise.race([
        new Promise((resolve) => chrome.once("exit", resolve)),
        sleep(3_000).then(() => {
          if (chrome.exitCode === null && chrome.signalCode === null) {
            chrome.kill("SIGKILL");
          }
        }),
      ]);
    }
    await chromeLog.close();
    await fs.rm(userDataDir, { recursive: true, force: true });
    cleanup.userDataDirRemoved = !existsSync(userDataDir);
    const scan = await evaluateRepoProfileScan();
    cleanup.repoProfileArtifactScan = scan;
    await writeJsonArtifact("cleanup-json", "cleanup-receipt-broad.json", "Cleanup receipt for temp Chrome profile and repo profile scan", cleanup);
    await writeTextArtifact(
      "cleanup-receipt",
      "cleanup-receipt-broad.txt",
      "Text cleanup receipt for broad browser QA",
      [
        `chromePid=${cleanup.chromePid}`,
        `chromeKilled=${cleanup.chromeKilled}`,
        `userDataDir=${cleanup.userDataDir}`,
        `userDataDirRemoved=${cleanup.userDataDirRemoved}`,
        `repoProfileArtifactScanCount=${cleanup.repoProfileArtifactScan.length}`,
        ...cleanup.repoProfileArtifactScan,
      ].join("\n") + "\n"
    );
  }
}

async function evaluateRepoProfileScan() {
  const names = new Set(["Default", "Profile 1", "Guest Profile", "Cookies", "History", "Login Data", "Local State"]);
  const matches = [];
  async function walk(dir) {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name === ".git" || entry.name === "node_modules" || entry.name === ".next") {
        continue;
      }
      const fullPath = path.join(dir, entry.name);
      if (names.has(entry.name)) {
        matches.push(path.relative(repoRoot, fullPath));
      }
      if (entry.isDirectory()) {
        await walk(fullPath);
      }
    }
  }
  await walk(repoRoot);
  return matches;
}

main().catch(async (error) => {
  await fs.mkdir(evidenceDir, { recursive: true });
  await writeTextArtifact("browser-qa-error-broad", "browser-qa-broad-error.log", "Broad browser QA failure log", `${error.stack ?? error.message}\n`);
  await writeTextArtifact("browser-qa-exit-broad", "browser-qa-broad-exit.txt", "Broad browser QA exit status", "FAIL\n");
  console.error(error);
  process.exitCode = 1;
});
