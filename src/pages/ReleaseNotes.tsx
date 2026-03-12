import { ExternalLink, Tag } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

interface Release {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  html_url: string;
}

export function ReleaseNotes() {
  const [releases, setReleases] = useState<Release[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchReleases = useCallback(async () => {
    try {
      const response = await fetch(
        "https://api.github.com/repos/CandleKeepAgents/candlekeep-desktop/releases?per_page=10"
      );
      if (response.ok) {
        const data = await response.json();
        setReleases(data);
      }
    } catch (err) {
      console.error("Failed to fetch releases:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchReleases();
  }, [fetchReleases]);

  if (loading) {
    return (
      <div className="space-y-4">
        <h2 className="text-lg font-semibold text-zinc-100">Release Notes</h2>
        <div className="animate-pulse space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-20 bg-zinc-800/50 rounded-lg" />
          ))}
        </div>
      </div>
    );
  }

  if (releases.length === 0) {
    return (
      <div className="space-y-4">
        <h2 className="text-lg font-semibold text-zinc-100">Release Notes</h2>
        <div className="p-6 text-center text-sm text-zinc-400 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
          No releases yet. Stay tuned!
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold text-zinc-100">Release Notes</h2>
      <div className="space-y-3">
        {releases.map((release) => (
          <div
            key={release.tag_name}
            className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <Tag className="w-4 h-4 text-amber-500" />
                <h3 className="text-sm font-medium text-zinc-100">
                  {release.name || release.tag_name}
                </h3>
              </div>
              <span className="text-xs text-zinc-500">
                {new Date(release.published_at).toLocaleDateString()}
              </span>
            </div>
            <p className="text-xs text-zinc-400 whitespace-pre-wrap line-clamp-4">
              {release.body || "No description"}
            </p>
            <a
              href={release.html_url}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 mt-2 text-xs text-amber-500 hover:text-amber-400"
            >
              View on GitHub
              <ExternalLink className="w-3 h-3" />
            </a>
          </div>
        ))}
      </div>
    </div>
  );
}
