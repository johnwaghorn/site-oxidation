import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import type { components } from "../src/generated/schema";
import {
  MADDIE_PASSWORD,
  TEMP_USER_PASSWORD,
  channelRow,
  openTeamNotifications,
  signIn,
  signInAsMaddie,
} from "./helpers";

type SiteExportResponse = components["schemas"]["SiteExportResponse"];

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
    await expect(
      teamList.getByText("Team Rocket", { exact: true }),
    ).toBeVisible();
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

test("maddie turns off email alerts", async ({ page }) => {
  await test.step("Given maddie has email alerts set up for Support", async () => {
    await signInAsMaddie(page);
    await openTeamNotifications(page, "Support");
    await page.getByRole("button", { name: "Set up Email" }).click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel("SMTP host").fill("smtp.waghorn.tech");
    await dialog
      .getByRole("switch", { name: "Sign in with a username and password" })
      .uncheck();
    await dialog.getByLabel("From address").fill("alerts@waghorn.tech");
    await dialog.getByLabel("To address").fill("maddie@waghorn.tech");
    await dialog.getByRole("button", { name: "Save" }).click();
    await expect(dialog).toBeHidden();
    await expect(
      channelRow(page, "Email").getByText("On", { exact: true }),
    ).toBeVisible();
  });
  await test.step("When she turns email off", async () => {
    await page.getByRole("button", { name: "Edit Email" }).click();
    const dialog = page.getByRole("dialog");
    await dialog.getByRole("button", { name: "Turn off email" }).click();
    await expect(dialog).toBeHidden();
  });
  await test.step("Then email shows as off for Support", async () => {
    await expect(
      channelRow(page, "Email").getByText("Off", { exact: true }),
    ).toBeVisible();
  });
});

for (const channel of ["Slack", "Microsoft Teams"]) {
  test(`maddie turns off ${channel} alerts`, async ({ page }) => {
    await test.step(`Given maddie has ${channel} alerts set up for Team Rocket`, async () => {
      await signInAsMaddie(page);
      await openTeamNotifications(page, "Team Rocket");
      await page.getByRole("button", { name: `Set up ${channel}` }).click();
      const dialog = page.getByRole("dialog");
      await dialog
        .getByRole("textbox")
        .fill("https://hooks.waghorn.tech/team-rocket");
      await dialog.getByRole("button", { name: "Save" }).click();
      await expect(dialog).toBeHidden();
      await expect(
        channelRow(page, channel).getByText("On", { exact: true }),
      ).toBeVisible();
    });
    await test.step(`When she turns ${channel} off`, async () => {
      await page.getByRole("button", { name: `Edit ${channel}` }).click();
      const dialog = page.getByRole("dialog");
      await dialog.getByRole("button", { name: `Turn off ${channel}` }).click();
      await expect(dialog).toBeHidden();
    });
    await test.step(`Then ${channel} shows as off for Team Rocket`, async () => {
      await expect(
        channelRow(page, channel).getByText("Off", { exact: true }),
      ).toBeVisible();
    });
  });
}

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

test("maddie exports her sites", async ({ page }) => {
  await test.step("Given maddie is signed in", async () => {
    await signInAsMaddie(page);
  });

  const download = await test.step("When she exports her sites", async () => {
    await page.getByRole("link", { name: "Import & Export" }).click();
    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export sites" }).click();
    return downloadPromise;
  });

  await test.step("Then the file holds her team's site settings", async () => {
    expect(download.suggestedFilename()).toMatch(
      /^site-oxidation-sites-\d{4}-\d{2}-\d{2}\.json$/,
    );
    const file: SiteExportResponse = JSON.parse(
      await readFile(await download.path(), "utf8"),
    );
    expect(file.version).toBe(1);
    expect(file.sites).toContainEqual(
      expect.objectContaining({
        name: "Maddie Blog",
        url: "https://maddie.waghorn.tech",
      }),
    );
  });
});

test("maddie re-imports her own export", async ({ page }) => {
  await test.step("Given maddie is on the import page", async () => {
    await signInAsMaddie(page);
    await page.getByRole("link", { name: "Import & Export" }).click();
  });
  const download = await test.step("When she exports her sites", async () => {
    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export sites" }).click();
    return downloadPromise;
  });
  await test.step("And imports that file into Team Rocket", async () => {
    await page.getByRole("combobox").selectOption({ label: "Team Rocket" });
    await page
      .locator('input[type="file"]')
      .setInputFiles(await download.path());
  });
  await test.step("Then her blog is reported as already there", async () => {
    await page.getByText(/^Show all \d+ sites?$/).click();
    const row = page.locator("li", { hasText: "Maddie Blog" });
    await expect(row.getByText("Already here", { exact: true })).toBeVisible();
  });
});
