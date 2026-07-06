import { expect, type Page } from "@playwright/test";

export const ADMIN_PASSWORD = "e2e-adminPassword-1234";
export const MADDIE_PASSWORD = "e2e-maddiePassword-1234";

export async function signIn(page: Page, username: string, password: string) {
  await page.goto("/");
  await page.getByPlaceholder("Username").fill(username);
  await page.getByPlaceholder("Password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Login" }).click();
}

export async function signInAsAdmin(page: Page) {
  await signIn(page, "admin", ADMIN_PASSWORD);
  await expect(page.getByRole("heading", { name: "Sites" })).toBeVisible();
}

export async function signInAsMaddie(page: Page) {
  await signIn(page, "maddie", MADDIE_PASSWORD);
  await expect(page.getByRole("heading", { name: "Sites" })).toBeVisible();
}
