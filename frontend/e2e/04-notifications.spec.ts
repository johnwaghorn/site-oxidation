import { expect, test } from "@playwright/test";
import {
  ensureTeamMembership,
  expandNotificationCard,
  signInAsAdmin,
} from "./helpers";

test.beforeEach(async ({ page }) => {
  await signInAsAdmin(page);
  await ensureTeamMembership(page, "Ops");
});

test("save a Slack webhook", async ({ page }) => {
  await test.step("When I open the Ops notification card", async () => {
    await page.goto("/notifications");
    await expandNotificationCard(page, "Ops");
  });

  await test.step("And I save a Slack webhook", async () => {
    const slackForm = page
      .locator("form")
      .filter({ hasText: "Slack webhook URL" });
    await slackForm
      .getByPlaceholder("https://hooks.slack.com/services/...")
      .fill("https://hooks.slack.test/services/e2e");
    await slackForm.getByRole("button", { name: "Save webhook" }).click();
  });

  await test.step("Then the Ops card shows Slack alerts are enabled", async () => {
    const card = page.locator("section").filter({ hasText: "Ops" });
    await expect(
      card.getByText("Slack alerts are enabled.", { exact: true }),
    ).toBeVisible();
    await expect(card.getByText("Enabled", { exact: true })).toBeVisible();
  });

  await test.step("And the saved webhook survives a reload", async () => {
    await page.reload();
    await expandNotificationCard(page, "Ops");
    const slackForm = page
      .locator("form")
      .filter({ hasText: "Slack webhook URL" });
    await expect(
      slackForm.getByPlaceholder("https://hooks.slack.com/services/..."),
    ).toHaveValue("https://hooks.slack.test/services/e2e");
  });
});

test("turn off the site recovered alert", async ({ page }) => {
  await test.step("When I open the Ops notification card", async () => {
    await page.goto("/notifications");
    await expandNotificationCard(page, "Ops");
  });

  await test.step("And I turn off the site recovered alert", async () => {
    const checkbox = page.getByLabel("Site recovered");
    await expect(checkbox).toBeChecked();
    await checkbox.click();
    await expect(checkbox).not.toBeChecked();
  });

  await test.step("Then it stays off after a reload", async () => {
    await page.reload();
    await expandNotificationCard(page, "Ops");
    await expect(page.getByLabel("Site recovered")).not.toBeChecked();
  });
});
