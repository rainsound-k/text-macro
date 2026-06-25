import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { Macro } from "../types";
import "./Settings.css";

type EditState = { id: string; title: string; content: string } | null;

// ── Hotkey capture ──────────────────────────────────────────────────────────

const MODIFIER_CODES = new Set([
  "ControlLeft", "ControlRight",
  "AltLeft", "AltRight",
  "ShiftLeft", "ShiftRight",
  "MetaLeft", "MetaRight",
  "OSLeft", "OSRight",
]);

function formatCode(code: string): string {
  if (code === "Space") return "Space";
  if (code.startsWith("Key")) return code.slice(3);       // KeyA → A
  if (code.startsWith("Digit")) return code.slice(5);     // Digit1 → 1
  if (/^F\d{1,2}$/.test(code)) return code;              // F1–F12
  const map: Record<string, string> = {
    Enter: "Enter", Tab: "Tab", Escape: "Escape",
    Backspace: "Backspace", Delete: "Delete",
    Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown",
    ArrowUp: "ArrowUp", ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft", ArrowRight: "ArrowRight",
  };
  return map[code] ?? "";
}

function HotkeyCapture({
  current,
  onSave,
}: {
  current: string;
  onSave: (key: string) => void;
}) {
  const [capturing, setCapturing] = useState(false);
  const [captured, setCaptured] = useState("");
  const captureRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (capturing) captureRef.current?.focus();
  }, [capturing]);

  function handleKeyDown(e: React.KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (MODIFIER_CODES.has(e.code)) return;

    const mods: string[] = [];
    if (e.ctrlKey) mods.push("Ctrl");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");
    if (e.metaKey) mods.push("Super");

    const key = formatCode(e.code);
    if (!key) return;
    setCaptured([...mods, key].join("+"));
  }

  if (!capturing) {
    return (
      <div className="hotkey-row">
        <code className="hotkey-badge">{current}</code>
        <button
          className="btn-ghost"
          onClick={() => { setCapturing(true); setCaptured(""); }}
        >
          변경
        </button>
      </div>
    );
  }

  return (
    <div
      ref={captureRef}
      className="hotkey-capture"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onBlur={() => setCapturing(false)}
    >
      {captured
        ? <code className="hotkey-badge">{captured}</code>
        : <span className="hotkey-capture-hint">새 단축키를 누르세요...</span>
      }
      <div className="hotkey-capture-actions">
        <button
          className="btn-primary btn-sm"
          disabled={!captured}
          onClick={() => { onSave(captured); setCapturing(false); }}
          onMouseDown={(e) => e.preventDefault()}
        >
          저장
        </button>
        <button
          className="btn-ghost"
          onClick={() => setCapturing(false)}
          onMouseDown={(e) => e.preventDefault()}
        >
          취소
        </button>
      </div>
    </div>
  );
}

// ── Main settings component ─────────────────────────────────────────────────

export default function Settings() {
  const [macros, setMacros] = useState<Macro[]>([]);
  const [edit, setEdit] = useState<EditState>(null);
  const [isAdding, setIsAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newContent, setNewContent] = useState("");
  const [hotkey, setHotkey] = useState("Alt+Space");
  const [autostart, setAutostart] = useState(false);
  const [status, setStatus] = useState("");
  const [accessibilityGranted, setAccessibilityGranted] = useState(true);

  useEffect(() => {
    api.getMacros().then(setMacros);
    api.getSettings().then((s) => setHotkey(s.hotkey));
    api.getAutostart().then(setAutostart);
    api.checkAccessibility().then(setAccessibilityGranted);
  }, []);

  function flash(msg: string) {
    setStatus(msg);
    setTimeout(() => setStatus(""), 2500);
  }

  async function handleAdd() {
    if (!newTitle.trim()) return;
    const added = await api.addMacro(newTitle.trim(), newContent.trim());
    setMacros((prev) => [...prev, added]);
    setNewTitle(""); setNewContent(""); setIsAdding(false);
    flash("매크로가 추가되었습니다");
  }

  async function handleUpdate() {
    if (!edit || !edit.title.trim()) return;
    await api.updateMacro(edit.id, edit.title.trim(), edit.content.trim());
    setMacros((prev) =>
      prev.map((m) => m.id === edit.id ? { ...m, ...edit } : m)
    );
    setEdit(null);
    flash("저장되었습니다");
  }

  async function handleDelete(id: string) {
    await api.deleteMacro(id);
    setMacros((prev) => prev.filter((m) => m.id !== id));
    if (edit?.id === id) setEdit(null);
    flash("삭제되었습니다");
  }

  async function handleMoveUp(idx: number) {
    if (idx === 0) return;
    const next = [...macros];
    [next[idx - 1], next[idx]] = [next[idx], next[idx - 1]];
    setMacros(next);
    await api.reorderMacros(next.map((m) => m.id));
  }

  async function handleMoveDown(idx: number) {
    if (idx === macros.length - 1) return;
    const next = [...macros];
    [next[idx], next[idx + 1]] = [next[idx + 1], next[idx]];
    setMacros(next);
    await api.reorderMacros(next.map((m) => m.id));
  }

  async function handleHotkeySave(newKey: string) {
    try {
      await api.updateHotkey(newKey);
      setHotkey(newKey);
      flash(`단축키가 ${newKey}(으)로 변경되었습니다`);
    } catch (e) {
      flash(`오류: ${e}`);
    }
  }

  async function handleAutostartToggle() {
    const next = !autostart;
    try {
      await api.setAutostart(next);
      setAutostart(next);
    } catch (e) {
      flash(`오류: ${e}`);
    }
  }

  return (
    <div className="settings">
      <aside className="settings-sidebar">
        <div className="settings-sidebar-header">
          <h2>매크로</h2>
          <button
            className="btn-icon"
            onClick={() => { setIsAdding(true); setEdit(null); }}
            title="추가"
          >
            +
          </button>
        </div>

        <ul className="macro-list">
          {macros.map((m, i) => (
            <li
              key={m.id}
              className={`macro-list-item ${edit?.id === m.id ? "active" : ""}`}
              onClick={() => { setEdit({ id: m.id, title: m.title, content: m.content }); setIsAdding(false); }}
            >
              <span className="macro-list-title">{m.title}</span>
              <span className="macro-list-preview">{m.content}</span>
              <div className="macro-list-actions">
                <button className="btn-icon-sm" onClick={(e) => { e.stopPropagation(); handleMoveUp(i); }}>↑</button>
                <button className="btn-icon-sm" onClick={(e) => { e.stopPropagation(); handleMoveDown(i); }}>↓</button>
                <button className="btn-icon-sm danger" onClick={(e) => { e.stopPropagation(); handleDelete(m.id); }}>✕</button>
              </div>
            </li>
          ))}
          {macros.length === 0 && (
            <li className="macro-list-empty">매크로가 없습니다</li>
          )}
        </ul>

        <div className="settings-io">
          <button className="btn-secondary" onClick={async () => { const r = await api.importMacros(); setMacros(r); flash(`가져오기 완료 (총 ${r.length}개)`); }}>
            가져오기
          </button>
          <button className="btn-secondary" onClick={async () => { await api.exportMacros(); flash("내보내기 완료"); }}>
            내보내기
          </button>
        </div>
      </aside>

      <main className="settings-main">
        {!accessibilityGranted && (
          <div className="permission-banner">
            <div className="permission-banner-text">
              <strong>손쉬운 사용 권한 필요</strong>
              <span>권한이 없으면 텍스트 붙여넣기가 작동하지 않습니다.</span>
            </div>
            <button
              className="permission-banner-btn"
              onClick={() => api.openAccessibilitySettings()}
            >
              설정 열기
            </button>
          </div>
        )}

        {isAdding && (
          <div className="edit-panel">
            <h3>새 매크로 추가</h3>
            <label>제목</label>
            <input className="edit-input" placeholder="예: 인사말" value={newTitle} onChange={(e) => setNewTitle(e.target.value)} autoFocus />
            <label>내용</label>
            <textarea className="edit-textarea" placeholder="붙여넣을 텍스트를 입력하세요" value={newContent} onChange={(e) => setNewContent(e.target.value)} rows={6} />
            <div className="edit-actions">
              <button className="btn-primary" onClick={handleAdd}>추가</button>
              <button className="btn-ghost" onClick={() => setIsAdding(false)}>취소</button>
            </div>
          </div>
        )}

        {edit && !isAdding && (
          <div className="edit-panel">
            <h3>매크로 편집</h3>
            <label>제목</label>
            <input className="edit-input" value={edit.title} onChange={(e) => setEdit({ ...edit, title: e.target.value })} autoFocus />
            <label>내용</label>
            <textarea className="edit-textarea" value={edit.content} onChange={(e) => setEdit({ ...edit, content: e.target.value })} rows={8} />
            <div className="edit-actions">
              <button className="btn-primary" onClick={handleUpdate}>저장</button>
              <button className="btn-ghost" onClick={() => setEdit(null)}>취소</button>
            </div>
          </div>
        )}

        {!edit && !isAdding && (
          <div className="system-panel">
            <h3>시스템 설정</h3>

            <div className="system-row">
              <div className="system-row-label">
                <span>전역 단축키</span>
                <span className="system-row-desc">어디서나 피커를 여는 단축키</span>
              </div>
              <HotkeyCapture current={hotkey} onSave={handleHotkeySave} />
            </div>

            <div className="system-row">
              <div className="system-row-label">
                <span>로그인 시 자동 실행</span>
                <span className="system-row-desc">컴퓨터 시작 시 자동으로 실행</span>
              </div>
              <button
                className={`toggle ${autostart ? "on" : ""}`}
                onClick={handleAutostartToggle}
                role="switch"
                aria-checked={autostart}
              />
            </div>
          </div>
        )}

        {status && <div className="settings-status">{status}</div>}
      </main>
    </div>
  );
}
