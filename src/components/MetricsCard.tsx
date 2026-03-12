import { BookOpen, Crown, Mail } from "lucide-react";
import type { WhoamiResponse } from "../lib/types";

interface MetricsCardProps {
  whoami: WhoamiResponse | null;
  loading: boolean;
}

export function MetricsCard({ whoami, loading }: MetricsCardProps) {
  if (loading) {
    return (
      <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50 animate-pulse">
        <div className="h-4 bg-zinc-700 rounded w-24 mb-3" />
        <div className="h-3 bg-zinc-700 rounded w-32" />
      </div>
    );
  }

  if (!whoami) {
    return (
      <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
        <p className="text-sm text-zinc-400">Sign in to see your library metrics</p>
      </div>
    );
  }

  const usage = whoami.item_count != null && whoami.item_limit != null
    ? `${whoami.item_count} / ${whoami.item_limit}`
    : whoami.item_count != null
    ? `${whoami.item_count}`
    : "—";

  return (
    <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50 space-y-3">
      <h3 className="text-sm font-medium text-zinc-300">Your Library</h3>
      <div className="grid grid-cols-2 gap-3">
        <div className="flex items-center gap-2">
          <BookOpen className="w-4 h-4 text-amber-500" />
          <div>
            <p className="text-lg font-semibold text-zinc-100">{usage}</p>
            <p className="text-xs text-zinc-500">Items</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Crown className="w-4 h-4 text-amber-500" />
          <div>
            <p className="text-lg font-semibold text-zinc-100 capitalize">{whoami.tier}</p>
            <p className="text-xs text-zinc-500">Tier</p>
          </div>
        </div>
      </div>
      <div className="flex items-center gap-2 pt-1 border-t border-zinc-700/50">
        <Mail className="w-3.5 h-3.5 text-zinc-500" />
        <p className="text-xs text-zinc-500">{whoami.email}</p>
      </div>
    </div>
  );
}
