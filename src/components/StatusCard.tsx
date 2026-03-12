import { AlertTriangle, CheckCircle, Loader2, XCircle } from "lucide-react";

interface StatusCardProps {
  title: string;
  status: "ok" | "error" | "loading" | "warning";
  detail?: string;
  action?: { label: string; onClick: () => void };
}

export function StatusCard({ title, status, detail, action }: StatusCardProps) {
  const icons = {
    ok: <CheckCircle className="w-5 h-5 text-green-500" />,
    error: <XCircle className="w-5 h-5 text-red-500" />,
    loading: <Loader2 className="w-5 h-5 text-blue-500 animate-spin" />,
    warning: <AlertTriangle className="w-5 h-5 text-amber-500" />,
  };

  return (
    <div className="flex items-center justify-between p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
      <div className="flex items-center gap-3">
        {icons[status]}
        <div>
          <p className="text-sm font-medium text-zinc-100">{title}</p>
          {detail && (
            <p className="text-xs text-zinc-400">{detail}</p>
          )}
        </div>
      </div>
      {action && (
        <button
          type="button"
          onClick={action.onClick}
          className="text-xs px-3 py-1.5 rounded-md bg-amber-600 hover:bg-amber-500 text-white transition-colors"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
