import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  BookOpen,
  FileText,
  LayoutDashboard,
  Puzzle,
  Settings as SettingsIcon,
  X,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Toaster } from "sonner";
import { checkAuthStatus, checkCliInstalled } from "./lib/tauri-commands";
import { Dashboard } from "./pages/Dashboard";
import { Integrations } from "./pages/Integrations";
import { ReleaseNotes } from "./pages/ReleaseNotes";
import { Settings } from "./pages/Settings";
import { Setup } from "./pages/Setup";

type Page = "dashboard" | "integrations" | "releases" | "settings";

function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [needsSetup, setNeedsSetup] = useState<boolean | null>(null);

  const checkSetupNeeded = useCallback(async () => {
    try {
      const [cli, auth] = await Promise.all([
        checkCliInstalled(),
        checkAuthStatus(),
      ]);
      setNeedsSetup(!cli.installed || !auth.authenticated);
    } catch {
      setNeedsSetup(true);
    }
  }, []);

  useEffect(() => {
    checkSetupNeeded();
  }, [checkSetupNeeded]);

  if (needsSetup === null) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-zinc-900">
        <div className="w-8 h-8 border-2 border-amber-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (needsSetup) {
    return (
      <div className="min-h-screen bg-zinc-900 text-zinc-100">
        <Setup onComplete={() => setNeedsSetup(false)} />
      </div>
    );
  }

  const navItems = [
    { id: "dashboard" as Page, icon: LayoutDashboard, label: "Home" },
    { id: "integrations" as Page, icon: Puzzle, label: "Integrations" },
    { id: "releases" as Page, icon: FileText, label: "Releases" },
    { id: "settings" as Page, icon: SettingsIcon, label: "Settings" },
  ];

  return (
    <div className="flex flex-col h-screen bg-zinc-900 text-zinc-100">
      <Toaster theme="dark" position="top-center" richColors />
      {/* Header — sticky, draggable */}
      {/* biome-ignore lint/a11y/noStaticElementInteractions: header uses onMouseDown for Tauri window dragging */}
      <header
        onMouseDown={async (e) => {
          // Only drag when clicking the header background, not buttons
          if ((e.target as HTMLElement).closest("button")) return;
          e.preventDefault();
          try { await getCurrentWindow().startDragging(); }
          catch (err) { console.error("drag failed:", err); }
        }}
        className="sticky top-0 z-10 flex items-center gap-2 px-4 py-3 bg-zinc-900 border-b border-zinc-800/50 cursor-default select-none shrink-0"
      >
        <BookOpen className="w-5 h-5 text-amber-500 pointer-events-none" />
        <span className="text-sm font-semibold text-zinc-200 flex-1 pointer-events-none">CandleKeep</span>
        <button
          type="button"
          onClick={async () => {
            try { await getCurrentWindow().hide(); }
            catch (e) { console.error("hide failed:", e); }
          }}
          className="text-zinc-500 hover:text-zinc-300 transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </header>

      {/* Content — scrollable, NOT draggable */}
      <main className="flex-1 overflow-y-auto p-4 min-h-0">
        {page === "dashboard" && <Dashboard />}
        {page === "integrations" && <Integrations />}
        {page === "releases" && <ReleaseNotes />}
        {page === "settings" && <Settings onLogout={() => setNeedsSetup(true)} />}
      </main>

      {/* Bottom Nav */}
      <nav className="flex items-center justify-around px-2 py-2 border-t border-zinc-800 bg-zinc-900 shrink-0">
        {navItems.map((item) => (
          <button
            type="button"
            key={item.id}
            onClick={() => setPage(item.id)}
            className={`flex flex-col items-center gap-0.5 px-3 py-1.5 rounded-lg transition-colors ${
              page === item.id
                ? "text-amber-500"
                : "text-zinc-500 hover:text-zinc-300"
            }`}
          >
            <item.icon className="w-4 h-4" />
            <span className="text-[10px]">{item.label}</span>
          </button>
        ))}
      </nav>
    </div>
  );
}

export default App;
