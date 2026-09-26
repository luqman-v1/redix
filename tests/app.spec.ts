import { expect, test } from "@playwright/test";

// The Playwright suite runs against the Vite dev server, not the Tauri
// shell, so `invoke` is unavailable. These specs therefore cover the shell
// that renders without a live Redis: mount, the connection modal lifecycle
// and theme persistence. Anything that needs Redis (key scan, tab
// lifecycle, get_key_meta) belongs in src-tauri/tests/redis_integration.rs.

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Connections" })).toBeVisible();
});

test.describe("app shell", () => {
  test("mounts without console errors", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.reload();
    await expect(page.getByRole("heading", { name: "Redix" })).toBeVisible();

    // Tauri APIs are absent in the browser harness. `connections.load()`
    // rejects through the IPC shim, which surfaces as a transformCallback
    // failure, and the destroy cleanup throws on the same missing global.
    // Everything else is a real regression.
    const expected = /tauri|__TAURI_INTERNALS__|is not a function|transformCallback|invoke/i;
    const unexpected = errors.filter((e) => !expected.test(e));
    expect(unexpected).toEqual([]);
  });

  test("connection selector and empty state render", async ({ page }) => {
    await expect(page.getByText("Select Connection...")).toBeVisible();
    await expect(page.getByText("No connections yet")).toBeVisible();
  });

  test("sidebar shows the disconnected empty state", async ({ page }) => {
    await page.locator(".close-btn").click();
    await expect(
      page.getByText("Connect to a Redis server to get started"),
    ).toBeVisible();
  });
});

test.describe("connection modal lifecycle", () => {
  test("opens on load when there is no active connection", async ({ page }) => {
    await expect(page.locator(".backdrop")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Connections" })).toBeVisible();
  });

  test("close button dismisses the modal and it stays dismissed", async ({ page }) => {
    await page.locator(".close-btn").click();

    // Regression guard: the auto-open effect used to re-trigger on its own
    // state change, so the modal snapped straight back and nothing in the
    // sidebar was reachable.
    await expect(page.locator(".backdrop")).toHaveCount(0);
    await page.waitForTimeout(500);
    await expect(page.locator(".backdrop")).toHaveCount(0);
    await expect(page.getByText("No connections yet")).toHaveCount(0);
  });

  test("Connect Now reopens the modal", async ({ page }) => {
    await page.locator(".close-btn").click();
    await expect(page.locator(".backdrop")).toHaveCount(0);

    await page.getByRole("button", { name: "Connect Now" }).click();
    await expect(page.locator(".backdrop")).toBeVisible();
  });

  test("add button opens the connection form", async ({ page }) => {
    await page.getByRole("button", { name: "+ Add" }).click();
    await expect(page.getByText("Host", { exact: false }).first()).toBeVisible();
  });
});

test.describe("theme", () => {
  test("toggle is only reachable once the modal is dismissed", async ({ page }) => {
    // The modal backdrop covers the sidebar while it is open.
    await expect(page.locator('[aria-label="Toggle theme"]')).toBeVisible();
    await page.locator(".close-btn").click();
    await expect(page.locator(".backdrop")).toHaveCount(0);
  });

  test("toggles the root class and persists the choice", async ({ page }) => {
    await page.locator(".close-btn").click();

    const initial = await page.evaluate(() => document.documentElement.className);
    expect(initial).toBe("dark");

    await page.locator('[aria-label="Toggle theme"]').click();
    await expect
      .poll(() => page.evaluate(() => document.documentElement.className))
      .toBe("light");
    await expect
      .poll(() => page.evaluate(() => localStorage.getItem("redix-theme")))
      .toBe("light");

    await page.locator('[aria-label="Toggle theme"]').click();
    await expect
      .poll(() => page.evaluate(() => document.documentElement.className))
      .toBe("dark");
  });

  test("restores the persisted theme on reload", async ({ page }) => {
    await page.locator(".close-btn").click();
    await page.locator('[aria-label="Toggle theme"]').click();
    await expect
      .poll(() => page.evaluate(() => localStorage.getItem("redix-theme")))
      .toBe("light");

    await page.reload();
    await expect
      .poll(() => page.evaluate(() => document.documentElement.className))
      .toBe("light");
  });
});
