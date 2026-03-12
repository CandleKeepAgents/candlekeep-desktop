import { ArrowUpCircle, Loader2 } from "lucide-react";

interface UpdateBannerProps {
  label?: string;
  currentVersion: string;
  latestVersion: string;
  onUpdate: () => void;
  loading?: boolean;
}

export function UpdateBanner({ label, currentVersion, latestVersion, onUpdate, loading }: UpdateBannerProps) {
  return (
    <div className="flex items-center justify-between p-3 rounded-lg bg-amber-900/30 border border-amber-700/50">
      <div className="flex items-center gap-2">
        <ArrowUpCircle className="w-5 h-5 text-amber-400" />
        <div>
          <p className="text-sm font-medium text-amber-200">{label ?? "Update Available"}</p>
          <p className="text-xs text-amber-400/70">{currentVersion} → {latestVersion}</p>
        </div>
      </div>
      <button
        type="button"
        onClick={onUpdate}
        disabled={loading}
        className="text-xs px-3 py-1.5 rounded-md bg-amber-600 hover:bg-amber-500 text-white transition-colors disabled:opacity-50 flex items-center gap-1.5"
      >
        {loading ? (
          <>
            <Loader2 className="w-3 h-3 animate-spin" />
            Updating...
          </>
        ) : (
          "Update"
        )}
      </button>
    </div>
  );
}
