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
import type { SetupStep, SystemCheckResult } from "../lib/types";

interface StepState {
  status: "pending" | "running" | "success" | "error";
  message: string;
}

export function Setup({ onComplete }: { onComplete: () => void }) {
  const [currentStep, setCurrentStep] = useState<SetupStep>("welcome");
  const [systemCheck, setSystemCheck] = useState<SystemCheckResult | null>(null);
  const [stepState, setStepState] = useState<StepState>({
    status: "pending",
    message: "",
  });
  const authPollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const authTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Cleanup auth polling on unmount
  useEffect(() => {
    return () => {
      if (authPollRef.current) clearInterval(authPollRef.current);
      if (authTimeoutRef.current) clearTimeout(authTimeoutRef.current);
    };
  }, []);

  const runSystemCheck = useCallback(async () => {
    setStepState({ status: "running", message: "Checking your system..." });
    try {
      const [homebrew, cargo, node, xcodeClt, cli, auth, claude, plugin] =
        await Promise.all([
          cmd.checkHomebrew(),
          cmd.checkCargo(),
          cmd.checkNode(),
          cmd.checkXcodeClt(),
          cmd.checkCliInstalled(),
          cmd.checkAuthStatus(),
          cmd.checkClaudeCodeInstalled(),
          cmd.checkPluginInstalled(),
        ]);

      const result: SystemCheckResult = {
        homebrew,
        cargo,
        node,
        xcodeClt,
        cliInstalled: cli.installed,
        cliVersion: cli.version,
        authenticated: auth.authenticated,
        claudeCodeInstalled: claude,
        pluginInstalled: plugin.installed,
        pluginVersion: plugin.version,
      };

      setSystemCheck(result);
      setStepState({ status: "success", message: "System check complete" });

      // Auto-advance to first needed step
      if (!result.homebrew) {
        setCurrentStep("install-homebrew");
      } else if (!result.cliInstalled) {
        setCurrentStep("install-cli");
      } else if (!result.authenticated) {
        setCurrentStep("authenticate");
      } else if (!result.claudeCodeInstalled) {
        setCurrentStep("install-claude");
      } else if (!result.pluginInstalled) {
        setCurrentStep("install-plugin");
      } else {
        setCurrentStep("done");
      }
    } catch (err) {
      setStepState({
        status: "error",
        message: `System check failed: ${err}`,
      });
    }
  }, []);

  const handleInstallHomebrew = async () => {
    setStepState({ status: "running", message: "Installing Homebrew... this may take a few minutes" });
    try {
      await cmd.installHomebrew();
      setStepState({ status: "success", message: "Homebrew installed!" });
      // Move to next step
      if (!systemCheck?.cliInstalled) {
        setCurrentStep("install-cli");
      } else {
        setCurrentStep("authenticate");
      }
    } catch (err) {
      setStepState({ status: "error", message: `${err}` });
    }
  };

  const handleInstallCli = async () => {
    setStepState({ status: "running", message: "Installing CandleKeep CLI..." });
    try {
      await cmd.installCli();
      setStepState({ status: "success", message: "CLI installed!" });
      setCurrentStep("authenticate");
    } catch (err) {
      setStepState({ status: "error", message: `${err}` });
    }
  };

  const handleAuth = async () => {
    if (stepState.status === "running") return; // Prevent double-click
    setStepState({ status: "running", message: "Opening browser for authentication..." });
    // Clear any previous polling
    if (authPollRef.current) clearInterval(authPollRef.current);
    if (authTimeoutRef.current) clearTimeout(authTimeoutRef.current);
    try {
      await cmd.triggerAuthLogin();
      setStepState({ status: "running", message: "Waiting for authentication... check your browser" });
      // Poll auth status
      authPollRef.current = setInterval(async () => {
        try {
          const auth = await cmd.checkAuthStatus();
          if (auth.authenticated) {
            if (authPollRef.current) clearInterval(authPollRef.current);
            if (authTimeoutRef.current) clearTimeout(authTimeoutRef.current);
            authPollRef.current = null;
            authTimeoutRef.current = null;
            setStepState({ status: "success", message: "Authenticated!" });
            if (!systemCheck?.claudeCodeInstalled) {
              setCurrentStep("install-claude");
            } else if (!systemCheck?.pluginInstalled) {
              setCurrentStep("install-plugin");
            } else {
              setCurrentStep("done");
            }
          }
        } catch {
          // Ignore individual poll errors, will retry
        }
      }, 2000);
      // Stop polling after 5 minutes with user feedback
      authTimeoutRef.current = setTimeout(() => {
        if (authPollRef.current) clearInterval(authPollRef.current);
        authPollRef.current = null;
        authTimeoutRef.current = null;
        setStepState({ status: "error", message: "Authentication timed out. Please try again." });
      }, 300000);
    } catch (err) {
      setStepState({ status: "error", message: `${err}` });
    }
  };

  const handleInstallPlugin = async () => {
    setStepState({ status: "running", message: "Installing CandleKeep plugin..." });
    try {
      await cmd.installPlugin();
      setStepState({ status: "success", message: "Plugin installed!" });
      setCurrentStep("done");
    } catch (err) {
      setStepState({ status: "error", message: `${err}` });
    }
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
            Let's set up everything you need to access your document library
            from your AI coding tools.
          </p>
        </div>
        <button
          type="button"
          onClick={() => {
            setCurrentStep("system-check");
            runSystemCheck();
          }}
          className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-amber-600 hover:bg-amber-500 text-white font-medium transition-colors"
        >
          Get Started
          <ArrowRight className="w-4 h-4" />
        </button>
      </div>
    );
  }

  if (currentStep === "done") {
    return (
      <div className="flex flex-col items-center justify-center min-h-[500px] text-center space-y-6 px-6">
        <div className="w-16 h-16 rounded-2xl bg-green-600/20 flex items-center justify-center">
          <Sparkles className="w-8 h-8 text-green-500" />
        </div>
        <div>
          <h1 className="text-xl font-bold text-zinc-100 mb-2">
            You're all set!
          </h1>
          <p className="text-sm text-zinc-400 max-w-xs">
            CandleKeep is ready to use. Your library is now accessible from
            Claude Code.
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

      {currentStep === "install-homebrew" && (
        <StepCard
          title="Install Homebrew"
          description="Homebrew is required to install the CandleKeep CLI. We'll install it for you."
          actionLabel="Install Homebrew"
          onAction={handleInstallHomebrew}
          disabled={stepState.status === "running"}
        />
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

      {currentStep === "install-claude" && (
        <StepCard
          title="Install Claude Code"
          description="Claude Code is required for the CandleKeep plugin. Visit claude.ai/download to install it, then come back."
          actionLabel="I've Installed Claude Code"
          onAction={async () => {
            const installed = await cmd.checkClaudeCodeInstalled();
            if (installed) {
              if (!systemCheck?.pluginInstalled) {
                setCurrentStep("install-plugin");
              } else {
                setCurrentStep("done");
              }
            } else {
              setStepState({
                status: "error",
                message: "Claude Code not detected. Please install it first.",
              });
            }
          }}
          disabled={stepState.status === "running"}
        />
      )}

      {currentStep === "install-plugin" && (
        <StepCard
          title="Install CandleKeep Plugin"
          description="This installs the CandleKeep plugin for Claude Code so you can access your library."
          actionLabel="Install Plugin"
          onAction={handleInstallPlugin}
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
