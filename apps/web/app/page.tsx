"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import type { FileDna } from "@genoma/shared-types";
import { DnaCanvas } from "@/features/visualization/DnaCanvas";
import { loadDemoDna } from "@/lib/api";

export default function LandingPage() {
  const [dna, setDna] = useState<FileDna | null>(null);

  useEffect(() => {
    loadDemoDna()
      .then(setDna)
      .catch(() => setDna(null));
  }, []);

  return (
    <main className="relative h-screen overflow-hidden bg-void">
      {dna ? (
        <div className="absolute inset-0 opacity-90">
          <DnaCanvas dna={dna} />
        </div>
      ) : (
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_40%,rgba(126,224,242,0.08),transparent_45%)]" />
      )}
      <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-void/20 via-transparent to-void/80" />
      <section className="relative z-10 flex h-full flex-col justify-end px-8 pb-16 md:px-16">
        <p className="font-mono text-[11px] tracking-[0.48em] text-cyan/80">GENOMA</p>
        <h1 className="mt-4 max-w-3xl text-4xl font-light tracking-[0.18em] md:text-6xl">
          THE DNA OF DIGITAL DATA
        </h1>
        <p className="mt-6 max-w-xl text-lg text-core/70">
          Every file has a structure.
          <br />
          We make it visible.
        </p>
        <div className="pointer-events-auto mt-10 flex flex-wrap gap-4">
          <Link
            href="/analyze"
            className="rounded-full bg-core px-6 py-3 text-sm tracking-[0.16em] text-void"
          >
            Analyze a File
          </Link>
          <Link
            href="/analyze?demo=sample.txt"
            className="rounded-full border border-white/15 px-6 py-3 text-sm tracking-[0.16em] text-core/80"
          >
            Explore Demo
          </Link>
        </div>
        <p className="mt-8 max-w-xl font-mono text-[11px] leading-5 text-core/35">
          GENOMA is an experimental structural fingerprinting engine. Digital DNA is not a
          cryptographic hash. π orients a deterministic representation of measured structure.
        </p>
      </section>
    </main>
  );
}
