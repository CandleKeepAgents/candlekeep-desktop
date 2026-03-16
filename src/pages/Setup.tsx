import {
  ArrowRight,
  BookOpen,
  CheckCircle,
  Loader2,
  Sparkles,
  XCircle,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import * as cmd from "../lib/tauri-commands";
import type {
  HostKind,
  PlatformInfo,
  SetupStep,
  SystemCheckResult,
} from "../lib/types";
import { HOST_DISPLAY_NAMES } from "../lib/types";

interface StepState {
  status: "pending" | "running" | "success" | "error";
  message: string;
}

export function Setup({ onComplete }: { onComplete: () => void }) {
  const [currentStep, setCurrentStep] = useState<SetupStep>("welcome");
  const [, setSystemCheck] = useState<SystemCheckResult | null>(null);
  const [selectedHosts, setSelectedHosts] = useState<HostKind[]>([]);
  const [, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [stepState, setStepState] = useState<StepState>({
    status: "pending",
    message: "",
  });
  const authPollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const authTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (authPollRef.current) clearInterval(authPollRef.current);
      if (authTimeoutRef.current) clearTimeout(authTimeoutRef.current);
    };
  }, []);

  // Load platform info on mount
  useEffect(() => {
    cmd.getPlatformInfo().then(setPlatformInfo).catch(console.error);
  }, []);

  const runSystemCheck = useCallback(async () => {
    setStepState({ status: "running", message: "Checking your system..." });
    try {
      const [platform, homebrew, cargo, node, xcodeClt, cli, auth, integrations] =
        await Promise.all([
          cmd.getPlatformInfo(),
          cmd.checkHomebrew(),
          cmd.checkCargo(),
          cmd.checkNode(),
          cmd.checkXcodeClt(),
          cmd.checkCliInstalled(),
          cmd.checkAuthStatus(),
          cmd.listIntegrations(),
        ]);

      setPlatformInfo(platform);

      const result: SystemCheckResult = {
        platform,
        homebrew,
        cargo,
        node,
        xcodeClt,
        cliInstalled: cli.installed,
        cliVersion: cli.version,
        authenticated: auth.authenticated,
        integrations,
      };

      setSystemCheck(result);
      setStepState({ status: "success", message: "System check complete" });

      // Auto-advance to first needed step
      if (!result.cliInstalled) {
        setCurrentStep("install-cli");
      } else if (!result.authenticated) {
        setCurrentStep("authenticate");
      } else if (selectedHosts.length > 0) {
        setCurrentStep("install-integration");
      } else {
        setCurrentStep("done");
      }
    } catch (err) {
      setStepState({
        status: "error",
        message: `System check failed: ${err}`,
      });
    }
  }, [selectedHosts]);

  const handleInstallCli = async () => {
    setStepState({
      status: "running",
      message: "Installing CandleKeep CLI...",
    });
    try {
      await cmd.installCli();
      setStepState({ status: "success", message: "CLI installed!" });
      setCurrentStep("authenticate");
    } catch (err) {
      setStepState({ status: "error", message: `${err}` });
    }
  };

  const handleAuth = async () => {
    if (stepState.status === "running") return;
    setStepState({
      status: "running",
      message: "Opening browser for authentication...",
    });
    if (authPollRef.current) clearInterval(authPollRef.current);
    if (authTimeoutRef.current) clearTimeout(authTimeoutRef.current);
    try {
      await cmd.triggerAuthLogin();
      setStepState({
        status: "running",
        message: "Waiting for authentication... check your browser",
      });
      authPollRef.current = setInterval(async () => {
        try {
          const auth = await cmd.checkAuthStatus();
          if (auth.authenticated) {
            if (authPollRef.current) clearInterval(authPollRef.current);
            if (authTimeoutRef.current) clearTimeout(authTimeoutRef.current);
            authPollRef.current = null;
            authTimeoutRef.current = null;
            setStepState({ status: "success", message: "Authenticated!" });

            if (selectedHosts.length > 0) {
              setCurrentStep("install-integration");
            } else {
              setCurrentStep("done");
            }
          }
        } catch {
          // Ignore poll errors
        }
      }, 2000);
      authTimeoutRef.current = setTimeout(() => {
        if (authPollRef.current) clearInterval(authPollRef.current);
        authPollRef.current = null;
        authTimeoutRef.current = null;
        setStepState({
          status: "error",
          message: "Authentication timed out. Please try again.",
        });
      }, 300000);
    } catch (err) {
      setStepState({ status: "error", message: `${err}` });
    }
  };

  const handleInstallIntegrations = async () => {
    if (selectedHosts.length === 0) return;
    const total = selectedHosts.length;
    const results: { host: HostKind; ok: boolean; message: string }[] = [];

    for (let i = 0; i < selectedHosts.length; i++) {
      const host = selectedHosts[i];
      setStepState({
        status: "running",
        message: `Installing CandleKeep for ${HOST_DISPLAY_NAMES[host]}... (${i + 1}/${total})`,
      });
      try {
        const result = await cmd.installIntegration(host);
        results.push({ host, ok: result.ok, message: result.message });
      } catch (err) {
        results.push({ host, ok: false, message: `${err}` });
      }
    }

    const succeeded = results.filter((r) => r.ok);
    const failed = results.filter((r) => !r.ok);

    if (failed.length === 0) {
      setStepState({
        status: "success",
        message: `Configured ${succeeded.length} integration${succeeded.length > 1 ? "s" : ""}`,
      });
    } else {
      setStepState({
        status: "error",
        message: `${succeeded.length} succeeded, ${failed.length} failed: ${failed.map((f) => `${HOST_DISPLAY_NAMES[f.host]}: ${f.message}`).join("; ")}`,
      });
    }
    setCurrentStep("done");
  };

  const stepIcon = (status: StepState["status"]) => {
    switch (status) {
      case "running":
        return <Loader2 className="w-5 h-5 text-amber-400 animate-spin" />;
      case "success":
        return <CheckCircle className="w-5 h-5 text-green-500" />;
      case "error":
        return <XCircle className="w-5 h-5 text-red-500" />;
      default:
        return <div className="w-5 h-5 rounded-full border-2 border-zinc-600" />;
    }
  };

  // --- Welcome screen ---
  if (currentStep === "welcome") {
    return (
      <div className="flex flex-col items-center justify-center min-h-[500px] text-center space-y-6 px-6">
        <div className="w-16 h-16 rounded-2xl bg-amber-600/20 flex items-center justify-center">
          <BookOpen className="w-8 h-8 text-amber-500" />
        </div>
        <div>
          <h1 className="text-xl font-bold text-zinc-100 mb-2">
            Welcome to CandleKeep
          </h1>
          <p className="text-sm text-zinc-400 max-w-xs">
            Let&apos;s set up everything you need to access your document
            library from your AI coding tools.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setCurrentStep("host-picker")}
          className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-amber-600 hover:bg-amber-500 text-white font-medium transition-colors"
        >
          Get Started
          <ArrowRight className="w-4 h-4" />
        </button>
      </div>
    );
  }

  // --- Host picker screen (multi-select) ---
  if (currentStep === "host-picker") {
    const hosts: HostKind[] = ["claude_code", "cursor", "codex", "amp", "open_code"];
    const toggleHost = (host: HostKind) => {
      setSelectedHosts((prev) =>
        prev.includes(host) ? prev.filter((h) => h !== host) : [...prev, host],
      );
    };
    return (
      <div className="space-y-6 px-2">
        <h2 className="text-lg font-semibold text-zinc-100">
          Choose Your AI Tools
        </h2>
        <p className="text-xs text-zinc-400">
          Select the AI coding tools you want to connect with CandleKeep. You
          can choose multiple.
        </p>
        <div className="space-y-2">
          {hosts.map((host) => {
            const selected = selectedHosts.includes(host);
            return (
              <button
                type="button"
                key={host}
                onClick={() => toggleHost(host)}
                className={`w-full p-4 rounded-lg border transition-colors text-left flex items-center gap-3 ${
                  selected
                    ? "border-amber-500 bg-amber-600/10"
                    : "border-zinc-700/50 bg-zinc-800/50 hover:border-zinc-600"
                }`}
              >
                <div
                  className={`w-5 h-5 rounded flex-shrink-0 border-2 flex items-center justify-center transition-colors ${
                    selected
                      ? "border-amber-500 bg-amber-600"
                      : "border-zinc-600 bg-transparent"
                  }`}
                >
                  {selected && (
                    <CheckCircle className="w-3.5 h-3.5 text-white" />
                  )}
                </div>
                <h4 className="text-sm font-medium text-zinc-100">
                  {HOST_DISPLAY_NAMES[host]}
                </h4>
              </button>
            );
          })}
        </div>
        <button
          type="button"
          onClick={() => {
            setCurrentStep("system-check");
            setTimeout(() => runSystemCheck(), 0);
          }}
          disabled={selectedHosts.length === 0}
          className="w-full flex items-center justify-center gap-2 px-6 py-2.5 rounded-lg bg-amber-600 hover:bg-amber-500 disabled:bg-zinc-700 disabled:text-zinc-500 text-white font-medium transition-colors"
        >
          Continue
          <ArrowRight className="w-4 h-4" />
        </button>
      </div>
    );
  }

  // --- Done screen ---
  if (currentStep === "done") {
    return (
      <div className="flex flex-col items-center justify-center min-h-[500px] text-center space-y-6 px-6">
        <div className="w-16 h-16 rounded-2xl bg-green-600/20 flex items-center justify-center">
          <Sparkles className="w-8 h-8 text-green-500" />
        </div>
        <div>
          <h1 className="text-xl font-bold text-zinc-100 mb-2">
            You&apos;re all set!
          </h1>
          <p className="text-sm text-zinc-400 max-w-xs">
            CandleKeep is ready to use.
            {selectedHosts.length > 0 &&
              ` Your library is now accessible from ${selectedHosts.map((h) => HOST_DISPLAY_NAMES[h]).join(", ")}.`}
          </p>
        </div>
        <button
          type="button"
          onClick={onComplete}
          className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-amber-600 hover:bg-amber-500 text-white font-medium transition-colors"
        >
          Go to Dashboard
          <ArrowRight className="w-4 h-4" />
        </button>
      </div>
    );
  }

  // --- Step-based screens ---
  return (
    <div className="space-y-6 px-2">
      <h2 className="text-lg font-semibold text-zinc-100">Setup</h2>

      <div className="flex items-center gap-2 p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
        {stepIcon(stepState.status)}
        <p className="text-sm text-zinc-300">{stepState.message || "Ready"}</p>
      </div>

      {currentStep === "system-check" && stepState.status === "running" && (
        <div className="text-center text-sm text-zinc-400 py-8">
          Checking system requirements...
        </div>
      )}

      {currentStep === "install-cli" && (
        <StepCard
          title="Install CandleKeep CLI"
          description="The CLI manages your library and handles authentication."
          actionLabel="Install CLI"
          onAction={handleInstallCli}
          disabled={stepState.status === "running"}
        />
      )}

      {currentStep === "authenticate" && (
        <StepCard
          title="Sign In"
          description="Authenticate with your CandleKeep account. A browser window will open."
          actionLabel="Open Browser to Sign In"
          onAction={handleAuth}
          disabled={stepState.status === "running"}
        />
      )}

      {currentStep === "install-integration" && selectedHosts.length > 0 && (
        <StepCard
          title="Connect Your AI Tools"
          description={`This will configure CandleKeep for ${selectedHosts.map((h) => HOST_DISPLAY_NAMES[h]).join(", ")}.`}
          actionLabel={`Install ${selectedHosts.length > 1 ? `${selectedHosts.length} Integrations` : "Integration"}`}
          onAction={handleInstallIntegrations}
          disabled={stepState.status === "running"}
        />
      )}

      {stepState.status === "error" && (
        <div className="p-3 rounded-lg bg-red-900/20 border border-red-800/50">
          <p className="text-xs text-red-300">{stepState.message}</p>
        </div>
      )}
    </div>
  );
}

function StepCard({
  title,
  description,
  actionLabel,
  onAction,
  disabled,
}: {
  title: string;
  description: string;
  actionLabel: string;
  onAction: () => void;
  disabled: boolean;
}) {
  return (
    <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50 space-y-3">
      <h3 className="text-sm font-medium text-zinc-100">{title}</h3>
      <p className="text-xs text-zinc-400">{description}</p>
      <button
        type="button"
        onClick={onAction}
        disabled={disabled}
        className="w-full text-sm px-4 py-2 rounded-lg bg-amber-600 hover:bg-amber-500 disabled:bg-zinc-700 disabled:text-zinc-500 text-white font-medium transition-colors"
      >
        {disabled ? (
          <span className="flex items-center justify-center gap-2">
            <Loader2 className="w-4 h-4 animate-spin" />
            Working...
          </span>
        ) : (
          actionLabel
        )}
      </button>
    </div>
  );
}
