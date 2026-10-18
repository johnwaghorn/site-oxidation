import { expect, type Page } from "@playwright/test";
import type { components, operations } from "../../src/generated/schema";

type ListTeamsResponse =
  operations["list_teams"]["responses"][200]["content"]["application/json"];
type ListUsersResponse =
  operations["list_users"]["responses"][200]["content"]["application/json"];
type TeamResponse = components["schemas"]["TeamResponse"];

export async function ensureTeamMembership(page: Page, name: string) {
  const teamsResponse = await page.request.get(
    `/api/admin/teams?search=${encodeURIComponent(name)}`,
  );
  expect(teamsResponse.ok()).toBe(true);
  const teams = (await teamsResponse.json()) as ListTeamsResponse;
  let team = teams.data.find((candidate) => candidate.name === name);
  if (!team) {
    const created = await page.request.post("/api/admin/teams", {
      data: { name },
    });
    expect(created.ok()).toBe(true);
    team = (await created.json()) as TeamResponse;
  }

  const usersResponse = await page.request.get("/api/admin/users?search=admin");
  expect(usersResponse.ok()).toBe(true);
  const users = (await usersResponse.json()) as ListUsersResponse;
  const admin = users.data.find((user) => user.username === "admin");
  expect(admin).toBeDefined();

  const added = await page.request.post(`/api/admin/teams/${team.id}/members`, {
    data: { user_id: admin!.id },
  });
  expect(added.ok() || added.status() === 409).toBe(true);
}
