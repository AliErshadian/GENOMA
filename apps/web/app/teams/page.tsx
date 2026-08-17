"use client";

import { FormEvent, useCallback, useEffect, useState } from "react";
import Link from "next/link";
import type { AnalysisSummary, Team, TeamMember } from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import {
  ApiError,
  addTeamMember,
  createTeam,
  getAuthToken,
  listTeamAnalyses,
  listTeamMembers,
  listTeams,
} from "@/lib/api";

export default function TeamsPage() {
  const [ready, setReady] = useState(false);
  const [teams, setTeams] = useState<Team[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [members, setMembers] = useState<TeamMember[]>([]);
  const [analyses, setAnalyses] = useState<AnalysisSummary[]>([]);
  const [name, setName] = useState("");
  const [inviteEmail, setInviteEmail] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [hasToken, setHasToken] = useState(false);

  const refreshTeams = useCallback(async () => {
    const items = await listTeams();
    setTeams(items);
    if (!selectedId && items[0]) setSelectedId(items[0].id);
  }, [selectedId]);

  useEffect(() => {
    setReady(true);
    setHasToken(Boolean(getAuthToken()));
  }, []);

  useEffect(() => {
    if (!ready || !hasToken) return;
    void refreshTeams().catch((err) =>
      setError(err instanceof ApiError ? err.message : "Failed to load teams"),
    );
  }, [ready, hasToken, refreshTeams]);

  useEffect(() => {
    if (!selectedId || !hasToken) {
      setMembers([]);
      setAnalyses([]);
      return;
    }
    void Promise.all([listTeamMembers(selectedId), listTeamAnalyses(selectedId)])
      .then(([nextMembers, nextAnalyses]) => {
        setMembers(nextMembers);
        setAnalyses(nextAnalyses);
      })
      .catch(() => {
        setMembers([]);
        setAnalyses([]);
      });
  }, [selectedId, hasToken]);

  async function onCreate(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const team = await createTeam(name);
      setName("");
      await refreshTeams();
      setSelectedId(team.id);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Create failed");
    } finally {
      setBusy(false);
    }
  }

  async function onInvite(event: FormEvent) {
    event.preventDefault();
    if (!selectedId) return;
    setBusy(true);
    setError(null);
    try {
      await addTeamMember(selectedId, inviteEmail);
      setInviteEmail("");
      setMembers(await listTeamMembers(selectedId));
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Invite failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <WorkspaceChrome>
      <div className="flex h-full items-start justify-center overflow-y-auto px-6 pb-24 pt-28 md:pl-56">
        <div className="w-full max-w-2xl">
          <p className="font-mono text-[11px] tracking-[0.32em] text-cyan">TEAMS</p>
          <h1 className="mt-3 font-mono text-xl tracking-[0.08em]">Collaboration</h1>
          <p className="mt-3 max-w-xl font-mono text-xs leading-relaxed text-core/50">
            Create teams, invite registered users by email, and share analyses for read access.
            Collaboration requires a signed-in session; set{" "}
            <span className="text-core/70">GENOMA_AUTH_REQUIRED=true</span> on the API when you want
            the server to enforce tokens.
          </p>

          {!ready ? null : !hasToken ? (
            <div className="mt-8 rounded-2xl border border-white/10 p-5 font-mono text-xs text-core/60">
              <p>You are not signed in.</p>
              <p className="mt-3">
                <Link href="/login" className="text-cyan hover:underline">
                  Sign in
                </Link>{" "}
                or{" "}
                <Link href="/register" className="text-cyan hover:underline">
                  register
                </Link>{" "}
                to manage teams. Auth stays optional for local demos unless the API requires it.
              </p>
            </div>
          ) : (
            <div className="mt-8 space-y-8">
              <form onSubmit={onCreate} className="flex flex-wrap items-end gap-3">
                <label className="block">
                  <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                    New team
                  </span>
                  <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    required
                    className="mt-2 block min-w-[220px] rounded-xl border border-white/10 bg-[#080a10] px-3 py-2 font-mono text-xs text-core outline-none focus:border-cyan/40"
                  />
                </label>
                <button
                  type="submit"
                  disabled={busy}
                  className="rounded-full border border-cyan/30 px-4 py-2 font-mono text-[11px] tracking-[0.14em] text-cyan hover:bg-cyan/10 disabled:opacity-35"
                >
                  Create
                </button>
              </form>

              <div>
                <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                  Your teams
                </p>
                {teams.length === 0 ? (
                  <p className="mt-2 font-mono text-xs text-core/45">No teams yet</p>
                ) : (
                  <ul className="mt-3 space-y-1">
                    {teams.map((team) => (
                      <li key={team.id}>
                        <button
                          type="button"
                          onClick={() => setSelectedId(team.id)}
                          className={`rounded-lg px-3 py-2 font-mono text-xs ${
                            selectedId === team.id
                              ? "bg-white/5 text-cyan"
                              : "text-core/70 hover:text-core"
                          }`}
                        >
                          {team.name}
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>

              {selectedId ? (
                <>
                  <form onSubmit={onInvite} className="flex flex-wrap items-end gap-3">
                    <label className="block">
                      <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                        Invite by email
                      </span>
                      <input
                        type="email"
                        value={inviteEmail}
                        onChange={(e) => setInviteEmail(e.target.value)}
                        required
                        className="mt-2 block min-w-[220px] rounded-xl border border-white/10 bg-[#080a10] px-3 py-2 font-mono text-xs text-core outline-none focus:border-cyan/40"
                      />
                    </label>
                    <button
                      type="submit"
                      disabled={busy}
                      className="rounded-full border border-white/15 px-4 py-2 font-mono text-[11px] tracking-[0.14em] text-core/80 hover:text-core disabled:opacity-35"
                    >
                      Invite
                    </button>
                  </form>

                  <div>
                    <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                      Members
                    </p>
                    <ul className="mt-2 space-y-1 font-mono text-xs text-core/70">
                      {members.map((member) => (
                        <li key={`${member.team_id}-${member.user_id}`}>
                          {member.email} · {member.role}
                        </li>
                      ))}
                    </ul>
                  </div>

                  <div>
                    <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                      Shared analyses
                    </p>
                    <ul className="mt-2 space-y-1">
                      {analyses.length === 0 ? (
                        <li className="font-mono text-xs text-core/45">None yet</li>
                      ) : (
                        analyses.map((item) => (
                          <li key={item.id}>
                            <Link
                              href={`/analyze/${item.id}`}
                              className="font-mono text-xs text-cyan hover:underline"
                            >
                              {item.original_name}
                            </Link>
                          </li>
                        ))
                      )}
                    </ul>
                  </div>
                </>
              ) : null}

              {error ? <p className="font-mono text-xs text-anomaly">{error}</p> : null}
            </div>
          )}
        </div>
      </div>
    </WorkspaceChrome>
  );
}
