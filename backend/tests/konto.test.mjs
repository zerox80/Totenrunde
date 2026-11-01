import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { setImmediate } from "node:timers/promises";
import test from "node:test";
import vm from "node:vm";

const source = await readFile(new URL("../web/konto/konto.js", import.meta.url), "utf8");

async function accountPage(signedIn) {
  const elements = new Map();
  const storage = new Map(signedIn ? [["totenrunde.session", "a".repeat(64)]] : []);
  const calls = [];
  function element(id) {
    if (!elements.has(id)) {
      elements.set(id, {
        hidden: true,
        disabled: false,
        value: "",
        textContent: "",
        listeners: {},
        addEventListener(type, listener) { this.listeners[type] = listener; },
        setAttribute() {},
        replaceChildren() {},
        append() {},
        reset() {},
        querySelector() { return element(`${id}-button`); },
      });
    }
    return elements.get(id);
  }
  const context = {
    URLSearchParams,
    document: {
      documentElement: {},
      querySelectorAll: () => [],
      getElementById: element,
      createElement: () => element(`created-${elements.size}`),
    },
    navigator: { languages: ["en"] },
    location: { search: "?link=ABCDEFGH", pathname: "/konto/" },
    history: { replaceState() {} },
    localStorage: {
      getItem: (key) => storage.get(key) ?? null,
      setItem: (key, value) => storage.set(key, value),
      removeItem: (key) => storage.delete(key),
    },
    async fetch(path, options) {
      calls.push({ path, ...options });
      let data;
      let status = 200;
      if (path === "/api/account") data = { name: "Anna", created: 1, stats: {} };
      else if (path === "/api/account/login" || path === "/api/account/register") {
        data = { name: "Anna", token: "a".repeat(64) };
      } else if (path === "/api/account/link") status = 204;
      else if (path === "/api/leaderboard") data = [];
      else if (path === "/api/mp/status") data = { online: 0, lobbies: 0 };
      else throw new Error(`Unexpected account request: ${path}`);
      return { ok: true, status, json: async () => data };
    },
  };
  await vm.runInNewContext(`(async () => {\n${source}\n})()`, context);
  return { element, calls, links: () => calls.filter((call) => call.path === "/api/account/link") };
}

test("signed-in player explicitly connects their game", async () => {
  const page = await accountPage(true);
  assert.equal(page.links().length, 0);
  assert.equal(page.element("confirm-link").hidden, false);
  assert.match(page.element("link-text").textContent, /Anna/);
  assert.equal(page.element("link-code").textContent, "ABCD-EFGH");

  await page.element("confirm-link").listeners.click();
  assert.equal(page.links().length, 1);
  assert.equal(page.links()[0].method, "POST");
  assert.deepEqual(JSON.parse(page.links()[0].body), { code: "ABCDEFGH" });
  assert.equal(page.element("link-done").hidden, false);
  assert.equal(page.element("confirm-link").hidden, true);
  await page.element("confirm-link").listeners.click();
  assert.equal(page.links().length, 1);
});

for (const form of ["login", "register"]) {
  test(`${form} displays device confirmation after signing in`, async () => {
    const page = await accountPage(false);
    assert.equal(page.element("confirm-link").hidden, true);
    page.element(form === "login" ? "login-name" : "reg-name").value = "Anna";
    page.element(form === "login" ? "login-pass" : "reg-pass").value = "geheim123";
    page.element("reg-pass2").value = "geheim123";
    page.element(`form-${form}`).listeners.submit({
      preventDefault() {},
      currentTarget: page.element(`form-${form}`),
    });
    await setImmediate();
    assert.equal(page.element("confirm-link").hidden, false);
    assert.equal(page.links().length, 0);
    await page.element("confirm-link").listeners.click();
    assert.equal(page.links().length, 1);
  });
}
