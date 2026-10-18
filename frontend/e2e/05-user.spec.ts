import { expect, test } from "@playwright/test";
import {
  MADDIE_PASSWORD,
  TEMP_USER_PASSWORD,
  channelRow,
  openTeamNotifications,
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
    await page.getByRole("button", { name: "Add site", exact: true }).click();
    await page.getByPlaceholder("Site name").fill("Maddie Blog");
    await page
      .getByPlaceholder("https://waghorn.tech")
      .fill("https://maddie.waghorn.tech");
    await page
      .getByTitle(/Team that owns this site/)
      .selectOption({ label: "Team Rocket" });
    await page.getByRole("button", { name: "Add Site", exact: true }).click();
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

  await test.step("Then she sees both of her teams", async () => {
    const teamList = page.getByRole("navigation", { name: "Teams" });
    await expect(teamList.getByText("Team Rocket", { exact: true })).toBeVisible();
    await expect(teamList.getByText("Support", { exact: true })).toBeVisible();
  });

  await test.step("When she saves a Slack webhook for Support", async () => {
    await openTeamNotifications(page, "Support");
    await page.getByRole("button", { name: "Set up Slack" }).click();
    const dialog = page.getByRole("dialog");
    await dialog
      .getByPlaceholder("https://hooks.slack.com/services/...")
      .fill("https://hooks.slack.test/services/support");
    await dialog.getByRole("button", { name: "Save" }).click();
    await expect(dialog).toBeHidden();
  });

  await test.step("Then Slack shows as on for Support", async () => {
    await expect(
      channelRow(page, "Slack").getByText("On", { exact: true }),
    ).toBeVisible();
  });
});

test("maddie checks which version is running", async ({ page }) => {
  await test.step("Given maddie is signed in", async () => {
    await signInAsMaddie(page);
  });

  await test.step("When she opens About from her account menu", async () => {
    await page.getByRole("button", { name: /maddie/ }).click();
    await page.getByRole("button", { name: "About" }).click();
  });

  await test.step("Then she sees the running version", async () => {
    const dialog = page.getByRole("dialog", { name: "About Site Oxidation" });
    await expect(dialog.getByText(/^Version: v\d+\.\d+\.\d+$/)).toBeVisible();
  });
});
