"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const NAV = [
  { href: "/analyze", label: "Analysis", ready: true },
  { href: "/analyze", label: "Explorer", ready: true },
  { href: "#", label: "Compare", ready: false },
  { href: "#", label: "Galaxy", ready: false },
  { href: "#", label: "Evolution", ready: false },
];

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
      <aside className="absolute left-5 top-24 z-20 hidden w-40 md:block">
        <nav className="panel space-y-1 rounded-2xl p-3">
          {NAV.map((item) => {
            const active = item.ready && pathname.startsWith("/analyze");
            return (
              <Link
                key={item.label}
                href={item.href}
                className={`block rounded-lg px-3 py-2 font-mono text-[11px] tracking-[0.14em] ${
                  item.label === "Analysis" && active
                    ? "bg-white/5 text-cyan"
                    : item.ready
                      ? "text-core/70 hover:text-core"
                      : "cursor-default text-core/25"
                }`}
              >
                {item.label}
                {!item.ready && <span className="ml-2 text-[9px] uppercase">Later</span>}
              </Link>
            );
          })}
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
