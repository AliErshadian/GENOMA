"use client";

import { useCallback, useState } from "react";

export function DropZone({ onFile }: { onFile: (file: File) => void }) {
  const [hover, setHover] = useState(false);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      setHover(false);
      const file = event.dataTransfer.files[0];
      if (file) onFile(file);
    },
    [onFile],
  );

  return (
    <label
      onDragOver={(event) => {
        event.preventDefault();
        setHover(true);
      }}
      onDragLeave={() => setHover(false)}
      onDrop={onDrop}
      className={`panel flex h-64 cursor-pointer flex-col items-center justify-center rounded-3xl border-dashed px-8 text-center transition ${
        hover ? "border-cyan/50 bg-cyan/5" : "border-white/10"
      }`}
    >
      <p className="text-sm tracking-[0.2em] text-core/80">DROP A FILE HERE</p>
      <p className="mt-3 max-w-sm font-mono text-xs leading-6 text-core/40">
        GENOMA streams the file, extracts structural features, and grows a Digital DNA organism.
        Files are treated as untrusted bytes and never executed.
      </p>
      <input
        type="file"
        className="hidden"
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) onFile(file);
        }}
      />
    </label>
  );
}
