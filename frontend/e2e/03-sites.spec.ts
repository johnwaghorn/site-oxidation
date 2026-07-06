import { expect, test } from "@playwright/test";
import { signInAsAdmin } from "./helpers";

test("add a site to the dashboard", async ({ page }) => {
  await test.step("Given I am signed in as the admin", async () => {
    await signInAsAdmin(page);
  });

  await test.step("When I add a site", async () => {
    await page.getByRole("button", { name: "Add Site" }).click();
    await page.getByPlaceholder("Site name").fill("Waghorn Tech");
    await page
      .getByPlaceholder("https://waghorn.tech")
      .fill("https://e2e.waghorn.tech");
    await page.getByRole("button", { name: "Add Site" }).click();
  });

  await test.step("Then the dashboard lists it as pending", async () => {
    const row = page.locator("tr", { hasText: "Waghorn Tech" });
    await expect(row).toBeVisible();
    await expect(row.getByText("PENDING", { exact: true })).toBeVisible();
  });
});
