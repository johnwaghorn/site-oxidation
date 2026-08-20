import { expect, test } from "@playwright/test";
import {
  addUserToTeamViaPanel,
  createTeamViaUi,
  createUserViaUi,
  signInAsAdmin,
} from "./helpers";

test.beforeEach(async ({ page }) => {
  await signInAsAdmin(page);
});

test("create a team", async ({ page }) => {
  await test.step("When I create a team named Team Rocket", async () => {
    await createTeamViaUi(page, "Team Rocket");
  });
});

test("create a user", async ({ page }) => {
  await test.step("When I create the user maddie in Team Rocket", async () => {
    await createUserViaUi(page, "maddie", "Team Rocket");
  });
});

test("add a user to a team via the members panel", async ({ page }) => {
  await test.step("Given a team named Support", async () => {
    await createTeamViaUi(page, "Support");
  });

  await test.step("When I add maddie to Support from the members panel", async () => {
    await addUserToTeamViaPanel(page, "Support", "maddie");
  });
});

test("save canary settings", async ({ page }) => {
  await test.step("When I open the Canary settings page", async () => {
    await page.goto("/admin/canary");
  });

  await test.step("And I enable the canary with a URL and timeout", async () => {
    await page.getByLabel("Canary enabled").check();
    await page
      .getByPlaceholder("https://waghorn.tech")
      .fill("http://127.0.0.1:8123/api/health");
    await page.getByLabel("Timeout in seconds").fill("5");
    await page.getByRole("button", { name: "Save changes" }).click();
  });

  await test.step("Then the settings survive a reload", async () => {
    await page.reload();
    await expect(page.getByLabel("Canary enabled")).toBeChecked();
    await expect(page.getByPlaceholder("https://waghorn.tech")).toHaveValue(
      "http://127.0.0.1:8123/api/health",
    );
    await expect(page.getByLabel("Timeout in seconds")).toHaveValue("5");
  });
});
