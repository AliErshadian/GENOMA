"use client";

import { useCallback, useState } from "react";

export function DropZone({
  onFile,
  onFiles,
}: {
  onFile?: (file: File) => void;
  onFiles?: (files: File[]) => void;
}) {
  const [hover, setHover] = useState(false);

  const emit = useCallback(
    (list: FileList | null) => {
      if (!list || list.length === 0) return;
      const files = Array.from(list);
      if (files.length === 1 && onFile) {
        const file = files[0];
        if (file) onFile(file);
        return;
      }
      if (onFiles) {
        onFiles(files);
        return;
      }
      if (onFile) {
        const file = files[0];
        if (file) onFile(file);
      }
    },
    [onFile, onFiles],
  );

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      setHover(false);
      emit(event.dataTransfer.files);
    },
    [emit],
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
      <p className="text-sm tracking-[0.2em] text-core/80">DROP FILE(S) HERE</p>
      <p className="mt-3 max-w-sm font-mono text-xs leading-6 text-core/40">
        GENOMA streams each file, extracts structural features, and grows Digital DNA. Multiple
        files upload sequentially. Bytes are treated as untrusted and never executed.
      </p>
      <input
        type="file"
        multiple
        className="hidden"
        onChange={(event) => {
          emit(event.target.files);
          event.target.value = "";
        }}
      />
    </label>
  );
}
