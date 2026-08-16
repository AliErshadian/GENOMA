"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import type { AnalysisSummary } from "@genoma/shared-types";
import { listAnalyses } from "@/lib/api";

const LATER = ["Compare", "Galaxy", "Evolution"] as const;

export function WorkspaceChrome({
  children,
  inspector,
  stats,
}: {
  children: React.ReactNode;
  inspector?: React.ReactNode;
  stats?: React.ReactNode;
}) {
  const pathname = usePathname();
  const [openExplorer, setOpenExplorer] = useState(pathname === "/analyze/explorer");
  const [analyses, setAnalyses] = useState<AnalysisSummary[]>([]);

  useEffect(() => {
    void listAnalyses()
      .then(setAnalyses)
      .catch(() => setAnalyses([]));
  }, [pathname]);

  return (
    <div className="relative h-screen w-screen overflow-hidden bg-void text-core">
      <header className="pointer-events-none absolute left-0 right-0 top-0 z-20 flex items-start justify-between p-6">
        <Link href="/" className="pointer-events-auto">
          <p className="text-[11px] tracking-[0.42em] text-cyan/80">GENOMA</p>
          <p className="mt-1 font-mono text-[10px] tracking-[0.18em] text-core/40">
            THE DNA OF DIGITAL DATA
          </p>
        </Link>
      </header>
      <aside className="absolute left-5 top-24 z-20 hidden w-44 md:block">
        <nav className="panel space-y-1 rounded-2xl p-3">
          <Link
            href="/analyze"
            className={`block rounded-lg px-3 py-2 font-mono text-[11px] tracking-[0.14em] ${
              pathname === "/analyze" ? "bg-white/5 text-cyan" : "text-core/70 hover:text-core"
            }`}
          >
            Analysis
          </Link>
          <button
            type="button"
            onClick={() => setOpenExplorer((value) => !value)}
            className={`block w-full rounded-lg px-3 py-2 text-left font-mono text-[11px] tracking-[0.14em] ${
              openExplorer || pathname.startsWith("/analyze/")
                ? "bg-white/5 text-cyan"
                : "text-core/70 hover:text-core"
            }`}
          >
            Explorer
          </button>
          {openExplorer ? (
            <ul className="max-h-48 space-y-1 overflow-y-auto px-1 pb-2">
              {analyses.length === 0 ? (
                <li className="px-2 py-1 font-mono text-[10px] text-core/35">No analyses yet</li>
              ) : (
                analyses.map((item) => (
                  <li key={item.id}>
                    <Link
                      href={`/analyze/${item.id}`}
                      className="block truncate rounded px-2 py-1 font-mono text-[10px] text-core/60 hover:text-core"
                    >
                      {item.original_name}
                    </Link>
                  </li>
                ))
              )}
            </ul>
          ) : null}
          {LATER.map((label) => (
            <span
              key={label}
              className="block cursor-default rounded-lg px-3 py-2 font-mono text-[11px] tracking-[0.14em] text-core/25"
            >
              {label}
              <span className="ml-2 text-[9px] uppercase">Later</span>
            </span>
          ))}
        </nav>
      </aside>
      <main className="absolute inset-0">{children}</main>
      {inspector ? (
        <aside className="absolute bottom-24 right-5 top-24 z-20 hidden w-72 lg:block">
          {inspector}
        </aside>
      ) : null}
      {stats ? (
        <footer className="absolute bottom-0 left-0 right-0 z-20 px-6 pb-5">{stats}</footer>
      ) : null}
    </div>
  );
}
