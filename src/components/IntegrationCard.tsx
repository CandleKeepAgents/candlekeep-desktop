import { ArrowUpCircle, CheckCircle, ChevronDown, ChevronUp, Clock, Download, Info, Loader2, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";

function friendlyError(raw: string): string {
  if (/failed to add marketplace/i.test(raw)) {
    return "Could not connect to the plugin marketplace. Check your internet.";
  }
  if (/not found|not installed/i.test(raw)) {
    return "Required tool not found. Make sure it's installed.";
  }
  return "Something went wrong.";
}

interface IntegrationCardProps {
  name: string;
  description: string;
  installed: boolean;
  version?: string | null;
  status: "installed" | "available" | "update-available" | "coming-soon";
  loading?: boolean;
  error?: string;
  setupHint?: string;
  successMessage?: string;
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
  error,
  setupHint,
  successMessage,
  onInstall,
  onUninstall,
  onUpdate,
}: IntegrationCardProps) {
  const [showErrorDetails, setShowErrorDetails] = useState(false);
  const [visibleSuccess, setVisibleSuccess] = useState<string | null>(null);

  useEffect(() => {
    if (successMessage) {
      setVisibleSuccess(successMessage);
      const timer = setTimeout(() => setVisibleSuccess(null), 3000);
      return () => clearTimeout(timer);
    }
    setVisibleSuccess(null);
  }, [successMessage]);

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

      {visibleSuccess && (
        <div className="flex items-center gap-1.5 p-2 mb-2 rounded-md bg-green-900/20 border border-green-800/50">
          <CheckCircle className="w-3.5 h-3.5 text-green-400 flex-shrink-0" />
          <p className="text-xs text-green-300">{visibleSuccess}</p>
        </div>
      )}

      {error && (
        <div className="p-2 mb-2 rounded-md bg-red-900/20 border border-red-800/50">
          <p className="text-xs text-red-300">{friendlyError(error)}</p>
          <button
            type="button"
            onClick={() => setShowErrorDetails((v) => !v)}
            className="flex items-center gap-1 mt-1 text-xs text-red-400/70 hover:text-red-300 transition-colors"
          >
            {showErrorDetails ? <ChevronUp className="w-3 h-3" /> : <ChevronDown className="w-3 h-3" />}
            {showErrorDetails ? "Hide details" : "Show details"}
          </button>
          {showErrorDetails && (
            <p className="mt-1.5 text-xs text-red-400/60 font-mono break-all">{error}</p>
          )}
        </div>
      )}

      {status === "available" && setupHint && !error && (
        <div className="flex items-start gap-1.5 p-2 mb-2 rounded-md bg-blue-900/10 border border-blue-800/30">
          <Info className="w-3.5 h-3.5 text-blue-400/70 flex-shrink-0 mt-0.5" />
          <p className="text-xs text-blue-300/70">{setupHint}</p>
        </div>
      )}

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
              Setting up...
            </>
          ) : (
            "Set up"
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
              Remove
            </>
          )}
        </button>
      )}
    </div>
  );
}
