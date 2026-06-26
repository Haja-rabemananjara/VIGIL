import { api } from "./api";

const LAST_TEAM_KEY = "vigil_last_team";

interface TeamView {
  id: string;
  name: string;
  role: string;
  created_at: string;
}

export async function postLoginDestination(token: string): Promise<string> {
  try {
    const teams = await api<TeamView[]>("/teams", { token });

    if (teams.length === 0) {
      return "/onboarding";
    }

    // TO DO LATER : uncomment when /teams/[teamId]/incidents exists
    // const lastTeamId = localStorage.getItem(LAST_TEAM_KEY);
    // const match = teams.find((t) => t.id === lastTeamId);
    // const target = match ?? teams[0];
    // return `/teams/${target.id}/incidents`;

    return "/onboarding";
  } catch {
    return "/onboarding";
  }
}

/** Saves the active team for next login. Called on team navigation. */
export function saveLastTeam(teamId: string): void {
  localStorage.setItem(LAST_TEAM_KEY, teamId);
}