import { test } from "@playwright/test";
import {
  addUserToTeamViaPanel,
  createTeamViaUi,
  createUserViaUi,
  signInAsAdmin,
} from "./helpers";

test.beforeEach(async ({ page }) => {
  await signInAsAdmin(page);
});

test("create a team", async ({ page }) => {
  await test.step("When I create a team named Team Rocket", async () => {
    await createTeamViaUi(page, "Team Rocket");
  });
});

test("create a user", async ({ page }) => {
  await test.step("When I create the user maddie in Team Rocket", async () => {
    await createUserViaUi(page, "maddie", "Team Rocket");
  });
});

test("add a user to a team via the members panel", async ({ page }) => {
  await test.step("Given a team named Support", async () => {
    await createTeamViaUi(page, "Support");
  });

  await test.step("When I add maddie to Support from the members panel", async () => {
    await addUserToTeamViaPanel(page, "Support", "maddie");
  });
});
