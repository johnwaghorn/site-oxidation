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

test("save SMTP email settings", async ({ page }) => {
  await test.step("When I open the Ops notification card", async () => {
    await page.goto("/notifications");
    await expandNotificationCard(page, "Ops");
  });

  await test.step("Then saving with only a host is rejected", async () => {
    const emailForm = page.locator("form").filter({ hasText: "SMTP host" });
    await emailForm
      .getByPlaceholder("smtp.waghorn.tech")
      .fill("mail.waghorn.tech");
    await emailForm
      .getByRole("button", { name: "Save email settings" })
      .click();
    await expect(
      emailForm.getByText(
        "Fill in the from address, to address, username and password to enable email notifications.",
      ),
    ).toBeVisible();
  });

  await test.step("And I save the SMTP email settings", async () => {
    const emailForm = page.locator("form").filter({ hasText: "SMTP host" });
    await emailForm.getByPlaceholder("Default for the TLS mode").fill("2525");
    await emailForm.getByLabel("TLS mode").selectOption("tls");
    await emailForm
      .getByPlaceholder("alerts@waghorn.tech")
      .fill("alerts@waghorn.tech");
    await emailForm
      .getByPlaceholder("on-call@waghorn.tech")
      .fill("john@waghorn.tech");
    await emailForm
      .getByLabel("Sign in with a username and password")
      .uncheck();
    await emailForm
      .getByRole("button", { name: "Save email settings" })
      .click();
  });

  await test.step("Then the Ops card shows email alerts are enabled", async () => {
    const card = page.locator("section").filter({ hasText: "Ops" });
    await expect(
      card.getByText("Slack, Email alerts are enabled.", { exact: true }),
    ).toBeVisible();
    await expect(
      card.getByRole("button", { name: "Send test email" }),
    ).toBeVisible();
  });

  await test.step("And the settings survive a reload", async () => {
    await page.reload();
    await expandNotificationCard(page, "Ops");
    const emailForm = page.locator("form").filter({ hasText: "SMTP host" });
    await expect(emailForm.getByPlaceholder("smtp.waghorn.tech")).toHaveValue(
      "mail.waghorn.tech",
    );
    await expect(
      emailForm.getByPlaceholder("Default for the TLS mode"),
    ).toHaveValue("2525");
    await expect(emailForm.getByLabel("TLS mode")).toHaveValue("tls");
    await expect(emailForm.getByPlaceholder("alerts@waghorn.tech")).toHaveValue(
      "alerts@waghorn.tech",
    );
    await expect(
      emailForm.getByPlaceholder("on-call@waghorn.tech"),
    ).toHaveValue("john@waghorn.tech");
    await expect(
      emailForm.getByLabel("Sign in with a username and password"),
    ).not.toBeChecked();
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
