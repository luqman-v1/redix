import { test, expect } from "@playwright/test";

test.describe("Redix App", () => {
  test("app loads and shows layout", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Connections")).toBeVisible();
    await expect(page.locator("text=No connections yet")).toBeVisible();
  });

  test("theme toggle exists and is clickable", async ({ page }) => {
    await page.goto("/");
    const toggle = page.locator('[aria-label="Toggle theme"]');
    await expect(toggle).toBeVisible();
    await toggle.click();
  });

  test("three panels are visible", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Connections")).toBeVisible();
    await expect(
      page.locator("text=Select a connection to browse keys"),
    ).toBeVisible();
    await expect(page.locator("text=Command console")).toBeVisible();
  });
});
