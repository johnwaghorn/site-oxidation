import { expect, test } from "@playwright/test";
import {
  MADDIE_PASSWORD,
  TEMP_USER_PASSWORD,
  expandNotificationCard,
  signIn,
  signInAsMaddie,
} from "./helpers";

test("maddie changes her temporary password on first sign-in", async ({
  page,
}) => {
  await test.step("When maddie signs in with her temporary password", async () => {
    await signIn(page, "maddie", TEMP_USER_PASSWORD);
  });

  await test.step("Then she is asked to change it", async () => {
    await expect(
      page.getByText("You must change your password before continuing."),
    ).toBeVisible();
  });

  await test.step("When she chooses a new password", async () => {
    await page.getByPlaceholder("Current password").fill(TEMP_USER_PASSWORD);
    await page
      .getByPlaceholder("New password (12+ characters)")
      .fill(MADDIE_PASSWORD);
    await page.getByPlaceholder("Confirm new password").fill(MADDIE_PASSWORD);
    await page.getByRole("button", { name: "Change Password" }).click();
  });

  await test.step("Then she sees the sites dashboard", async () => {
    await expect(page.getByRole("heading", { name: "Sites" })).toBeVisible();
  });
});

test("maddie adds a site for her team", async ({ page }) => {
  await test.step("Given maddie is signed in", async () => {
    await signInAsMaddie(page);
  });

  await test.step("When she adds a site owned by Team Rocket", async () => {
    await page.getByRole("button", { name: "Add Site" }).click();
    await page.getByPlaceholder("Site name").fill("Maddie Blog");
    await page
      .getByPlaceholder("https://waghorn.tech")
      .fill("https://maddie.waghorn.tech");
    await page
      .getByTitle(/Team that owns this site/)
      .selectOption({ label: "Team Rocket" });
    await page.getByRole("button", { name: "Add Site" }).click();
  });

  await test.step("Then the dashboard lists it as pending", async () => {
    const row = page.locator("tr", { hasText: "Maddie Blog" });
    await expect(row).toBeVisible();
    await expect(row.getByText("PENDING", { exact: true })).toBeVisible();
  });
});

test("maddie configures notifications for her team", async ({ page }) => {
  await test.step("Given maddie is signed in", async () => {
    await signInAsMaddie(page);
  });

  await test.step("When she opens the notifications page", async () => {
    await page.goto("/notifications");
  });

  await test.step("Then she sees cards for both of her teams", async () => {
    await expect(
      page.getByRole("button", { name: "Team Rocket" }),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Support" })).toBeVisible();
  });

  await test.step("When she saves a Slack webhook for Support", async () => {
    await expandNotificationCard(page, "Support");
    const slackForm = page
      .locator("form")
      .filter({ hasText: "Slack webhook URL" });
    await slackForm
      .getByPlaceholder("https://hooks.slack.com/services/...")
      .fill("https://hooks.slack.test/services/support");
    await slackForm.getByRole("button", { name: "Save webhook" }).click();
  });

  await test.step("Then the Support card shows Slack alerts are enabled", async () => {
    const card = page.locator("section").filter({ hasText: "Support" });
    await expect(
      card.getByText("Slack alerts are enabled.", { exact: true }),
    ).toBeVisible();
    await expect(card.getByText("Enabled", { exact: true })).toBeVisible();
  });
});
