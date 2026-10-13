import { Link, Outlet, useParams } from "react-router-dom";
import { useAdminTeam } from "../hooks/useAdmin";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Tabs } from "../components/ui/Tabs";
import { Truncate } from "../components/ui/Truncate";

export function AdminTeamDetail() {
  const { id } = useParams<{ id: string }>();
  const teamId = Number(id);
  const {
    data: team,
    isLoading: teamLoading,
    error: teamError,
  } = useAdminTeam(teamId);

  if (!Number.isInteger(teamId) || teamId <= 0) {
    return <ErrorMessage error={new Error("Team not found")} />;
  }
  if (teamLoading) return <LoadingSpinner />;
  if (teamError) return <ErrorMessage error={teamError} />;
  if (!team) return <ErrorMessage error={new Error("Team not found")} />;

  return (
    <div className="page-wrapper">
      <Link to="/admin/teams" className="back-link">
        &larr; Back to Teams
      </Link>

      <h1 className="page-title">
        <Truncate text={team.name} maxWidth="90%" />
      </h1>
      <p className="muted-text">
        {team.member_count} active member{team.member_count === 1 ? "" : "s"}{" "}
        and {team.site_count} assigned site{team.site_count === 1 ? "" : "s"}
      </p>

      <Tabs
        items={[
          { to: `/admin/teams/${teamId}/members`, label: "Members" },
          { to: `/admin/teams/${teamId}/sites`, label: "Sites" },
        ]}
      />
      <Outlet context={team} />
    </div>
  );
}
