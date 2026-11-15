import { expect, type Page } from "@playwright/test";

export async function openTeamNotifications(page: Page, teamName: string) {
  await page.goto("/notifications");
  const teamHeading = page.getByRole("heading", { name: teamName, level: 2 });
  const teamLink = page
    .getByRole("table")
    .getByRole("link", { name: teamName, exact: true });
  await expect(teamHeading.or(teamLink)).toBeVisible();
  if (await teamLink.isVisible()) {
    await teamLink.click();
  }
  await expect(teamHeading).toBeVisible();
}

export function channelRow(page: Page, channel: string) {
  return page
    .getByRole("listitem")
    .filter({ has: page.getByText(channel, { exact: true }) });
}
