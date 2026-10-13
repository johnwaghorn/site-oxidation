import { expect, test } from "@playwright/test";
import {
  channelRow,
  ensureTeamMembership,
  openTeamNotifications,
  signInAsAdmin,
} from "./helpers";

test.beforeEach(async ({ page }) => {
  await signInAsAdmin(page);
  await ensureTeamMembership(page, "Ops");
});

test("save a Slack webhook", async ({ page }) => {
  await test.step("When I open the Ops notifications", async () => {
    await openTeamNotifications(page, "Ops");
  });

  await test.step("And I save a Slack webhook", async () => {
    await page.getByRole("button", { name: "Set up Slack" }).click();
    const dialog = page.getByRole("dialog");
    await dialog
      .getByPlaceholder("https://hooks.slack.com/services/...")
      .fill("https://hooks.slack.test/services/e2e");
    await dialog.getByRole("button", { name: "Save" }).click();
    await expect(dialog).toBeHidden();
  });

  await test.step("Then the Slack channel shows as on", async () => {
    const row = channelRow(page, "Slack");
    await expect(row.getByText("On", { exact: true })).toBeVisible();
    await expect(row.getByText("Webhook saved")).toBeVisible();
  });

  await test.step("And the saved webhook survives a reload", async () => {
    await page.reload();
    await openTeamNotifications(page, "Ops");
    await page.getByRole("button", { name: "Edit Slack" }).click();
    await expect(
      page
        .getByRole("dialog")
        .getByPlaceholder("https://hooks.slack.com/services/..."),
    ).toHaveValue("https://hooks.slack.test/services/e2e");
  });
});

test("save SMTP email settings", async ({ page }) => {
  await test.step("When I open the Ops email settings", async () => {
    await openTeamNotifications(page, "Ops");
    await page.getByRole("button", { name: "Set up Email" }).click();
  });

  await test.step("Then saving with only a host is rejected", async () => {
    const dialog = page.getByRole("dialog");
    await dialog.getByPlaceholder("smtp.waghorn.tech").fill("mail.waghorn.tech");
    await dialog.getByRole("button", { name: "Save" }).click();
    await expect(
      dialog.getByText(
        "Fill in the from address, to address, username and password to enable email notifications.",
      ),
    ).toBeVisible();
  });

  await test.step("And I save the SMTP email settings", async () => {
    const dialog = page.getByRole("dialog");
    await dialog.getByPlaceholder("Default for the TLS mode").fill("2525");
    await dialog.getByLabel("TLS mode").selectOption("tls");
    await dialog
      .getByPlaceholder("alerts@waghorn.tech")
      .fill("alerts@waghorn.tech");
    await dialog
      .getByPlaceholder("on-call@waghorn.tech")
      .fill("john@waghorn.tech");
    await dialog.getByLabel("Sign in with a username and password").uncheck();
    await dialog.getByRole("button", { name: "Save" }).click();
    await expect(dialog).toBeHidden();
  });

  await test.step("Then the Email channel shows as on", async () => {
    const row = channelRow(page, "Email");
    await expect(row.getByText("On", { exact: true })).toBeVisible();
    await expect(
      row.getByText("mail.waghorn.tech to john@waghorn.tech"),
    ).toBeVisible();
    await expect(row.getByRole("button", { name: "Send test" })).toBeVisible();
  });

  await test.step("And the settings survive a reload", async () => {
    await page.reload();
    await openTeamNotifications(page, "Ops");
    await page.getByRole("button", { name: "Edit Email" }).click();
    const dialog = page.getByRole("dialog");
    await expect(dialog.getByPlaceholder("smtp.waghorn.tech")).toHaveValue(
      "mail.waghorn.tech",
    );
    await expect(dialog.getByPlaceholder("Default for the TLS mode")).toHaveValue(
      "2525",
    );
    await expect(dialog.getByLabel("TLS mode")).toHaveValue("tls");
    await expect(dialog.getByPlaceholder("alerts@waghorn.tech")).toHaveValue(
      "alerts@waghorn.tech",
    );
    await expect(dialog.getByPlaceholder("on-call@waghorn.tech")).toHaveValue(
      "john@waghorn.tech",
    );
    await expect(
      dialog.getByLabel("Sign in with a username and password"),
    ).not.toBeChecked();
  });
});

test("turn off the site recovered alert", async ({ page }) => {
  await test.step("When I open the Ops notifications", async () => {
    await openTeamNotifications(page, "Ops");
  });

  await test.step("And I turn off the site recovered alert", async () => {
    const toggle = page.getByLabel("Site recovered");
    await expect(toggle).toBeChecked();
    await toggle.click();
    await expect(toggle).not.toBeChecked();
  });

  await test.step("Then it stays off after a reload", async () => {
    await page.reload();
    await openTeamNotifications(page, "Ops");
    await expect(page.getByLabel("Site recovered")).not.toBeChecked();
  });
});
