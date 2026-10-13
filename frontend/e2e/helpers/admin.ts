import { expect, type Page } from "@playwright/test";

export const TEMP_USER_PASSWORD = "e2e-tempPassword-1234";

export async function createTeamViaUi(page: Page, name: string) {
  await page.goto("/admin/teams");
  await page.getByRole("button", { name: "Add team", exact: true }).click();
  await page.getByPlaceholder("New team name").fill(name);
  await page.getByRole("button", { name: "Create Team", exact: true }).click();
  await expect(page.getByRole("link", { name })).toBeVisible();
}

export async function createUserViaUi(
  page: Page,
  username: string,
  teamName: string,
) {
  await page.goto("/admin/users");
  await page.getByRole("button", { name: "Add user", exact: true }).click();
  await page.getByPlaceholder("e.g. jsmith").fill(username);
  await page.getByPlaceholder("12+ characters").fill(TEMP_USER_PASSWORD);
  await page.getByPlaceholder("Search teams...").fill(teamName);
  await page.getByRole("option", { name: teamName }).click();
  await expect(page.getByText("Team selected")).toBeVisible();
  await page.getByRole("button", { name: "Create User", exact: true }).click();
  await expect(page.locator("tr", { hasText: username })).toBeVisible();
}

export async function addUserToTeamViaTeamPage(
  page: Page,
  teamName: string,
  username: string,
) {
  await page.goto("/admin/teams");
  await page.getByRole("link", { name: teamName }).click();
  await page.getByRole("button", { name: "Add member" }).click();
  const dialog = page.getByRole("dialog");
  await dialog.getByPlaceholder("Search users to add...").fill(username);
  await dialog.getByRole("button", { name: `Add ${username}` }).click();
  await dialog.getByRole("button", { name: "Done" }).click();
  await expect(page.locator("tr", { hasText: username })).toBeVisible();
}
