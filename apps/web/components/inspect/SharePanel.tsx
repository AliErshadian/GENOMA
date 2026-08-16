"use client";

import { useEffect, useState } from "react";
import type { Team } from "@genoma/shared-types";
import { ApiError, getAuthToken, listTeams, shareAnalysis } from "@/lib/api";

export function SharePanel({ analysisId }: { analysisId: string }) {
  const [teams, setTeams] = useState<Team[]>([]);
  const [teamId, setTeamId] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const signedIn = Boolean(getAuthToken());

  useEffect(() => {
    if (!signedIn) return;
    void listTeams()
      .then((items) => {
        setTeams(items);
        if (items[0]) setTeamId(items[0].id);
      })
      .catch(() => setTeams([]));
  }, [signedIn]);

  if (!signedIn) {
    return (
      <div className="panel rounded-2xl p-3">
        <p className="font-mono text-[10px] uppercase tracking-[0.22em] text-core/40">Share</p>
        <p className="mt-2 font-mono text-[10px] text-core/45">
          Sign in and enable auth to share with a team.
        </p>
      </div>
    );
  }

  async function onShare() {
    if (!teamId) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      await shareAnalysis(analysisId, teamId);
      setMessage("Shared with team");
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Share failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="panel rounded-2xl p-3">
      <p className="font-mono text-[10px] uppercase tracking-[0.22em] text-core/40">Share</p>
      {teams.length === 0 ? (
        <p className="mt-2 font-mono text-[10px] text-core/45">
          Create a team on the Teams page first.
        </p>
      ) : (
        <>
          <select
            value={teamId}
            onChange={(e) => setTeamId(e.target.value)}
            className="mt-2 w-full rounded-lg border border-white/10 bg-[#080a10] px-2 py-1.5 font-mono text-[11px] text-core outline-none"
          >
            {teams.map((team) => (
              <option key={team.id} value={team.id}>
                {team.name}
              </option>
            ))}
          </select>
          <button
            type="button"
            disabled={busy || !teamId}
            onClick={() => void onShare()}
            className="mt-2 rounded-full border border-white/15 px-3 py-1 font-mono text-[10px] tracking-[0.12em] text-core/80 hover:text-core disabled:opacity-35"
          >
            {busy ? "Sharing…" : "Share with team"}
          </button>
        </>
      )}
      {message ? <p className="mt-2 font-mono text-[10px] text-cyan">{message}</p> : null}
      {error ? <p className="mt-2 font-mono text-[10px] text-anomaly">{error}</p> : null}
    </div>
  );
}
