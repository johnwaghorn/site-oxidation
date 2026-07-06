import { expect, type Page } from "@playwright/test";

export async function ensureTeamMembership(page: Page, name: string) {
  const teamsResponse = await page.request.get(
    `/api/admin/teams?search=${encodeURIComponent(name)}`,
  );
  expect(teamsResponse.ok()).toBe(true);
  const teams = (await teamsResponse.json()) as {
    data: { id: number; name: string }[];
  };
  let team = teams.data.find((candidate) => candidate.name === name);
  if (!team) {
    const created = await page.request.post("/api/admin/teams", {
      data: { name },
    });
    expect(created.ok()).toBe(true);
    team = (await created.json()) as { id: number; name: string };
  }

  const usersResponse = await page.request.get("/api/admin/users?search=admin");
  expect(usersResponse.ok()).toBe(true);
  const users = (await usersResponse.json()) as {
    data: { id: number; username: string }[];
  };
  const admin = users.data.find((user) => user.username === "admin");
  expect(admin).toBeDefined();

  const added = await page.request.post(`/api/admin/teams/${team.id}/members`, {
    data: { user_id: admin!.id },
  });
  expect(added.ok() || added.status() === 409).toBe(true);
}
