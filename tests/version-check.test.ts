import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { checkForUpdate, compareVersions } from "../src/lib/utils/version-check";

const CACHE_KEY = "redix_update_check";

const store = new Map<string, string>();
Object.defineProperty(globalThis, "localStorage", {
  configurable: true,
  value: {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => void store.set(k, v),
    removeItem: (k: string) => void store.delete(k),
  },
});

function release(overrides: Record<string, unknown> = {}) {
  return {
    tag_name: "v0.3.0",
    html_url: "https://github.com/luqmannulhakim/redix/releases/tag/v0.3.0",
    draft: false,
    prerelease: false,
    ...overrides,
  };
}

beforeEach(() => {
  store.clear();
  globalThis.fetch = (async () =>
    new Response(JSON.stringify(release()), {
      status: 200,
      headers: { "content-type": "application/json" },
    })) as unknown as typeof fetch;
});

afterEach(() => {
  store.clear();
});

describe("compareVersions", () => {
  test("orders by major, then minor, then patch", () => {
    expect(compareVersions("0.2.1", "0.2.2")).toBe(-1);
    expect(compareVersions("0.3.0", "0.2.9")).toBe(1);
    expect(compareVersions("1.0.0", "0.9.9")).toBe(1);
    expect(compareVersions("0.2.1", "0.2.1")).toBe(0);
  });

  test("tolerates a leading v and surrounding whitespace", () => {
    expect(compareVersions(" v0.3.0 ", "0.2.1")).toBe(1);
  });

  test("returns null for unparseable versions", () => {
    expect(compareVersions("", "0.2.1")).toBeNull();
    expect(compareVersions("0.2.1", "nightly")).toBeNull();
    expect(compareVersions("1.2", "1.2.1")).toBeNull();
  });
});

describe("checkForUpdate", () => {
  test("reports a newer release", async () => {
    const info = await checkForUpdate("0.2.1");
    expect(info).not.toBeNull();
    expect(info!.latest).toBe("0.3.0");
    expect(info!.url).toContain("/releases/tag/v0.3.0");
  });

  test("stays quiet when already on the latest version", async () => {
    expect(await checkForUpdate("0.3.0")).toBeNull();
  });

  test("stays quiet when running ahead of the release feed", async () => {
    expect(await checkForUpdate("0.4.0")).toBeNull();
  });

  test("ignores drafts and prereleases", async () => {
    globalThis.fetch = (async () =>
      new Response(JSON.stringify(release({ draft: true })), { status: 200 })) as unknown as typeof fetch;
    expect(await checkForUpdate("0.2.1")).toBeNull();

    store.clear();
    globalThis.fetch = (async () =>
      new Response(JSON.stringify(release({ prerelease: true })), { status: 200 })) as unknown as typeof fetch;
    expect(await checkForUpdate("0.2.1")).toBeNull();
  });

  test("returns null on a non-ok response instead of throwing", async () => {
    globalThis.fetch = (async () => new Response("rate limited", { status: 403 })) as unknown as typeof fetch;
    expect(await checkForUpdate("0.2.1")).toBeNull();
  });

  test("returns null when the network rejects", async () => {
    globalThis.fetch = (async () => {
      throw new Error("offline");
    }) as unknown as typeof fetch;
    expect(await checkForUpdate("0.2.1")).toBeNull();
  });

  test("caches a successful probe and reuses it within the TTL", async () => {
    let calls = 0;
    globalThis.fetch = (async () => {
      calls++;
      return new Response(JSON.stringify(release()), { status: 200 });
    }) as unknown as typeof fetch;

    const start = 1_000_000;
    await checkForUpdate("0.2.1", start);
    expect(calls).toBe(1);

    await checkForUpdate("0.2.1", start + 60_000);
    expect(calls).toBe(1);
    expect(store.has(CACHE_KEY)).toBe(true);

    // Past the TTL the endpoint is probed again.
    await checkForUpdate("0.2.1", start + 7 * 60 * 60 * 1000);
    expect(calls).toBe(2);
  });

  test("caches a failed probe so a dead endpoint is not retried every launch", async () => {
    let calls = 0;
    globalThis.fetch = (async () => {
      calls++;
      return new Response("nope", { status: 500 });
    }) as unknown as typeof fetch;

    const start = 1_000_000;
    expect(await checkForUpdate("0.2.1", start)).toBeNull();
    expect(await checkForUpdate("0.2.1", start + 1_000)).toBeNull();
    expect(calls).toBe(1);
  });
});
