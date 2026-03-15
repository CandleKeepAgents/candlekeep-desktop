import { ArrowUpCircle, CheckCircle, Clock, Download, Loader2, Trash2 } from "lucide-react";

interface IntegrationCardProps {
  name: string;
  description: string;
  installed: boolean;
  version?: string | null;
  status: "installed" | "available" | "update-available" | "coming-soon";
  loading?: boolean;
  onInstall?: () => void;
  onUninstall?: () => void;
  onUpdate?: () => void;
}

export function IntegrationCard({
  name,
  description,
  version,
  status,
  loading,
  onInstall,
  onUninstall,
  onUpdate,
}: IntegrationCardProps) {
  const statusBadge = {
    installed: (
      <span className="flex items-center gap-1 text-xs text-green-400">
        <CheckCircle className="w-3.5 h-3.5" /> Installed
      </span>
    ),
    available: (
      <span className="flex items-center gap-1 text-xs text-blue-400">
        <Download className="w-3.5 h-3.5" /> Available
      </span>
    ),
    "update-available": (
      <span className="flex items-center gap-1 text-xs text-amber-400">
        <ArrowUpCircle className="w-3.5 h-3.5" /> Update
      </span>
    ),
    "coming-soon": (
      <span className="flex items-center gap-1 text-xs text-zinc-500">
        <Clock className="w-3.5 h-3.5" /> Coming Soon
      </span>
    ),
  };

  return (
    <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
      <div className="flex items-start justify-between mb-2">
        <div>
          <h4 className="text-sm font-medium text-zinc-100">{name}</h4>
          {version && (
            <p className="text-xs text-zinc-500">v{version}</p>
          )}
        </div>
        {statusBadge[status]}
      </div>
      <p className="text-xs text-zinc-400 mb-3">{description}</p>
      {status === "available" && onInstall && (
        <button
          type="button"
          onClick={onInstall}
          disabled={loading}
          className="w-full text-xs px-3 py-1.5 rounded-md bg-amber-600 hover:bg-amber-500 text-white transition-colors flex items-center justify-center gap-1.5 disabled:bg-zinc-700 disabled:text-zinc-500"
        >
          {loading ? (
            <>
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Installing...
            </>
          ) : (
            "Install"
          )}
        </button>
      )}
      {status === "update-available" && onUpdate && (
        <button
          type="button"
          onClick={onUpdate}
          disabled={loading}
          className="w-full text-xs px-3 py-1.5 rounded-md bg-amber-600 hover:bg-amber-500 text-white transition-colors flex items-center justify-center gap-1.5 disabled:bg-zinc-700 disabled:text-zinc-500"
        >
          {loading ? (
            <>
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Updating...
            </>
          ) : (
            "Update"
          )}
        </button>
      )}
      {(status === "installed" || status === "update-available") && onUninstall && (
        <button
          type="button"
          onClick={onUninstall}
          disabled={loading}
          className="w-full text-xs px-3 py-1.5 mt-1.5 rounded-md bg-zinc-700 hover:bg-red-600/80 text-zinc-300 hover:text-white transition-colors flex items-center justify-center gap-1.5 disabled:bg-zinc-700 disabled:text-zinc-500"
        >
          {loading ? (
            <>
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Removing...
            </>
          ) : (
            <>
              <Trash2 className="w-3.5 h-3.5" />
              Uninstall
            </>
          )}
        </button>
      )}
    </div>
  );
}
