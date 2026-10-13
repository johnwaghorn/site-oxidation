import { expect, test } from "@playwright/test";
import {
  addUserToTeamViaTeamPage,
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

test("add a user to a team from the team page", async ({ page }) => {
  await test.step("Given a team named Support", async () => {
    await createTeamViaUi(page, "Support");
  });

  await test.step("When I add maddie to Support from the team page", async () => {
    await addUserToTeamViaTeamPage(page, "Support", "maddie");
  });
});

test("save canary settings", async ({ page }) => {
  await test.step("When I open the Canary settings page", async () => {
    await page.goto("/admin/canary");
  });

  await test.step("And I set a canary URL and timeout", async () => {
    await page.getByRole("button", { name: "Edit settings" }).click();
    const dialog = page.getByRole("dialog", { name: "Canary settings" });
    await dialog
      .getByLabel("Canary URL")
      .fill("http://127.0.0.1:8123/api/health");
    await dialog.getByLabel("Timeout in seconds").fill("5");
    await dialog.getByRole("button", { name: "Save" }).click();
    await expect(dialog).toBeHidden();
  });

  await test.step("And I turn on scheduled checks", async () => {
    const toggle = page.getByRole("switch", { name: "Scheduled checks" });
    await toggle.click();
    await expect(toggle).toBeChecked();
  });

  await test.step("Then the settings survive a reload", async () => {
    await page.reload();
    await expect(
      page.getByRole("switch", { name: "Scheduled checks" }),
    ).toBeChecked();
    await expect(
      page.getByRole("link", { name: /http:\/\/127\.0\.0\.1:8123\/api\/health/ }),
    ).toBeVisible();
    await expect(page.getByText("5 seconds", { exact: true })).toBeVisible();
  });
});
