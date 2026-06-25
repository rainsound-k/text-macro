import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../api";
import type { Macro } from "../types";
import "./MacroPicker.css";

const win = getCurrentWindow();

export default function MacroPicker() {
  const [macros, setMacros] = useState<Macro[]>([]);
  const [query, setQuery] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const filtered = macros.filter(
    (m) =>
      m.title.toLowerCase().includes(query.toLowerCase()) ||
      m.content.toLowerCase().includes(query.toLowerCase())
  );

  // Load macros on mount
  useEffect(() => {
    api.getMacros().then(setMacros);
  }, []);

  // Reset state and focus input whenever the window gains focus
  useEffect(() => {
    const unlisten = win.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        api.getMacros().then(setMacros);
        setQuery("");
        setSelectedIdx(0);
        setTimeout(() => inputRef.current?.focus(), 50);
      } else {
        win.hide();
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Keep selectedIdx in bounds when filter changes
  useEffect(() => {
    setSelectedIdx((i) => Math.min(i, Math.max(filtered.length - 1, 0)));
  }, [filtered.length]);

  // Prevent WebKit/macOS from intercepting ⌘1–9 before JS sees it
  useEffect(() => {
    const prevent = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && /^[1-9]$/.test(e.key)) {
        e.preventDefault();
      }
    };
    document.addEventListener("keydown", prevent, true);
    return () => document.removeEventListener("keydown", prevent, true);
  }, []);

  // Scroll selected item into view
  useEffect(() => {
    const item = listRef.current?.children[selectedIdx] as HTMLElement | undefined;
    item?.scrollIntoView({ block: "nearest" });
  }, [selectedIdx]);

  async function selectMacro(m: Macro) {
    await api.pasteText(m.content);
  }

  function onKeyDown(e: React.KeyboardEvent) {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIdx((i) => Math.min(i + 1, filtered.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIdx((i) => Math.max(i - 1, 0));
        break;
      case "Enter":
        e.preventDefault();
        if (filtered[selectedIdx]) selectMacro(filtered[selectedIdx]);
        break;
      case "Escape":
        win.hide();
        break;
    }

    // ⌘1–⌘9 / Ctrl+1–9 quick select
    if ((e.metaKey || e.ctrlKey) && e.key >= "1" && e.key <= "9") {
      e.preventDefault();
      e.stopPropagation();
      const idx = parseInt(e.key, 10) - 1;
      if (filtered[idx]) selectMacro(filtered[idx]);
    }
  }

  return (
    <div className="picker" onKeyDown={onKeyDown}>
      <div className="picker-search">
        <span className="picker-search-icon">⌕</span>
        <input
          ref={inputRef}
          className="picker-input"
          placeholder="매크로 검색..."
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setSelectedIdx(0);
          }}
          autoFocus
        />
      </div>

      {filtered.length === 0 ? (
        <div className="picker-empty">일치하는 매크로가 없습니다</div>
      ) : (
        <ul className="picker-list" ref={listRef}>
          {filtered.map((m, i) => (
            <li
              key={m.id}
              className={`picker-item ${i === selectedIdx ? "selected" : ""}`}
              onClick={() => selectMacro(m)}
              onMouseEnter={() => setSelectedIdx(i)}
            >
              <div className="picker-item-header">
                <span className="picker-item-title">{m.title}</span>
                {i < 9 && (
                  <span className="picker-item-shortcut">
                    {navigator.platform.includes("Mac") ? "⌘" : "Ctrl+"}
                    {i + 1}
                  </span>
                )}
              </div>
              <p className="picker-item-preview">{m.content}</p>
            </li>
          ))}
        </ul>
      )}

      <div className="picker-footer">
        <span>↑↓ 이동</span>
        <span>Enter 선택</span>
        <span>Esc 닫기</span>
        <button
          className="picker-settings-btn"
          onClick={() => {
            win.hide();
            api.showSettings();
          }}
        >
          설정
        </button>
      </div>
    </div>
  );
}
