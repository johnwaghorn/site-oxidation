import { expect, test } from "@playwright/test";
import { ADMIN_PASSWORD } from "./helpers";

test("first run: create the admin account and sign in", async ({ page }) => {
  let generatedPassword = "";

  await test.step("When I open the app", async () => {
    await page.goto("/");
  });

  await test.step("Then I am invited to create the admin account", async () => {
    await expect(
      page.getByRole("heading", { name: "Welcome to Site Oxidation" }),
    ).toBeVisible();
  });

  await test.step("When I create the admin account", async () => {
    await page.getByRole("button", { name: "Create admin user" }).click();
  });

  await test.step("Then I am shown the generated admin credentials", async () => {
    await expect(
      page.getByText("Save these now", { exact: false }),
    ).toBeVisible();
    generatedPassword = await page
      .getByTestId("generated-password")
      .innerText();
    expect(generatedPassword.length).toBeGreaterThan(10);
  });

  await test.step("When I continue to the login page", async () => {
    await page
      .getByRole("button", { name: "I saved my password, continue" })
      .click();
  });

  await test.step("And I sign in with the generated credentials", async () => {
    await page.getByPlaceholder("Username").fill("admin");
    await page
      .getByPlaceholder("Password", { exact: true })
      .fill(generatedPassword);
    await page.getByRole("button", { name: "Login" }).click();
  });

  await test.step("Then I am asked to change my password", async () => {
    await expect(
      page.getByText("You must change your password before continuing."),
    ).toBeVisible();
  });

  await test.step("When I change my password", async () => {
    await page.getByPlaceholder("Current password").fill(generatedPassword);
    await page
      .getByPlaceholder("New password (12+ characters)")
      .fill(ADMIN_PASSWORD);
    await page.getByPlaceholder("Confirm new password").fill(ADMIN_PASSWORD);
    await page.getByRole("button", { name: "Change Password" }).click();
  });

  await test.step("Then I see the sites dashboard", async () => {
    await expect(page.getByRole("heading", { name: "Sites" })).toBeVisible();
  });
});
