import { expect, type Page } from "@playwright/test";

export async function openTeamNotifications(page: Page, teamName: string) {
  await page.goto("/notifications");
  const teamList = page.getByRole("navigation", { name: "Teams" });
  if (await teamList.isVisible()) {
    await teamList
      .getByRole("link")
      .filter({ has: page.getByText(teamName, { exact: true }) })
      .click();
  }
  await expect(
    page.getByRole("heading", { name: teamName, level: 2 }),
  ).toBeVisible();
}

export function channelRow(page: Page, channel: string) {
  return page
    .getByRole("listitem")
    .filter({ has: page.getByText(channel, { exact: true }) });
}
