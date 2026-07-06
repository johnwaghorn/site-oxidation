import { expect, type Page } from "@playwright/test";

export async function expandNotificationCard(page: Page, teamName: string) {
  const disclosure = page.getByRole("button", { name: teamName });
  await expect(disclosure).toBeVisible();
  if ((await disclosure.getAttribute("aria-expanded")) === "false") {
    await disclosure.click();
  }
  await expect(disclosure).toHaveAttribute("aria-expanded", "true");
}
