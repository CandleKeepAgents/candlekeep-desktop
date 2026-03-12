import { ExternalLink, KeyRound, Loader2, RefreshCw } from "lucide-react";

interface QuickActionsProps {
  onOpenLibrary: () => void;
  onReAuth: () => void;
  onCheckUpdates: () => void;
  updatesLoading?: boolean;
}

export function QuickActions({ onOpenLibrary, onReAuth, onCheckUpdates, updatesLoading }: QuickActionsProps) {
  const actions = [
    { icon: ExternalLink, label: "Open Library", onClick: onOpenLibrary },
    { icon: KeyRound, label: "Re-authenticate", onClick: onReAuth },
    { icon: updatesLoading ? Loader2 : RefreshCw, label: updatesLoading ? "Checking..." : "Check Updates", onClick: onCheckUpdates, spinning: updatesLoading },
  ];

  return (
    <div className="space-y-2">
      <h3 className="text-sm font-medium text-zinc-300">Quick Actions</h3>
      <div className="grid grid-cols-3 gap-2">
        {actions.map((action) => (
          <button
            type="button"
            key={action.label}
            onClick={action.onClick}
            disabled={action.spinning}
            className="flex flex-col items-center gap-1.5 p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50 hover:bg-zinc-700/50 transition-colors disabled:opacity-50"
          >
            <action.icon className={`w-4 h-4 text-amber-500 ${action.spinning ? "animate-spin" : ""}`} />
            <span className="text-xs text-zinc-400">{action.label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
