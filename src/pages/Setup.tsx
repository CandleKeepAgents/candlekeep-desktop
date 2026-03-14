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
  const [systemCheck, setSystemCheck] = useState<SystemCheckResult | null>(
    null,
  );
  const [selectedHost, setSelectedHost] = useState<HostKind | null>(null);
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
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

  const isMacOS = platformInfo?.platform === "macos";

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
      const isMac = platform.platform === "macos";
      if (isMac && !result.homebrew) {
        setCurrentStep("install-homebrew");
      } else if (!result.cliInstalled) {
        setCurrentStep("install-cli");
      } else if (!result.authenticated) {
        setCurrentStep("authenticate");
      } else {
        // Check if selected host integration needs install
        const hostStatus = selectedHost
          ? result.integrations.find((i) => i.host === selectedHost)
          : null;
        if (hostStatus && !hostStatus.host_installed) {
          setCurrentStep("install-host");
        } else if (hostStatus && !hostStatus.integration_installed) {
          setCurrentStep("install-integration");
        } else {
          setCurrentStep("done");
        }
      }
    } catch (err) {
      setStepState({
        status: "error",
        message: `System check failed: ${err}`,
      });
    }
  }, [selectedHost]);

  const handleInstallHomebrew = async () => {
    setStepState({
      status: "running",
      message: "Installing Homebrew... this may take a few minutes",
    });
    try {
      await cmd.installHomebrew();
      setStepState({ status: "success", message: "Homebrew installed!" });
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

            // Check host integration status
            if (selectedHost) {
              const status = await cmd.checkIntegration(selectedHost);
              if (!status.host_installed) {
                setCurrentStep("install-host");
              } else if (!status.integration_installed) {
                setCurrentStep("install-integration");
              } else {
                setCurrentStep("done");
              }
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

  const handleInstallIntegration = async () => {
    if (!selectedHost) return;
    setStepState({
      status: "running",
      message: `Installing CandleKeep for ${HOST_DISPLAY_NAMES[selectedHost]}...`,
    });
    try {
      const result = await cmd.installIntegration(selectedHost);
      if (result.ok) {
        setStepState({ status: "success", message: result.message });
        setCurrentStep("done");
      } else {
        setStepState({ status: "error", message: result.message });
      }
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

  // --- Host picker screen ---
  if (currentStep === "host-picker") {
    const hosts: HostKind[] = ["claude_code", "cursor", "codex", "amp"];
    return (
      <div className="space-y-6 px-2">
        <h2 className="text-lg font-semibold text-zinc-100">
          Choose Your AI Tool
        </h2>
        <p className="text-xs text-zinc-400">
          Select the AI coding tool you want to connect with CandleKeep.
        </p>
        <div className="space-y-2">
          {hosts.map((host) => (
            <button
              type="button"
              key={host}
              onClick={() => {
                setSelectedHost(host);
                setCurrentStep("system-check");
                // Trigger system check after state update
                setTimeout(() => runSystemCheck(), 0);
              }}
              className={`w-full p-4 rounded-lg border transition-colors text-left ${
                selectedHost === host
                  ? "border-amber-500 bg-amber-600/10"
                  : "border-zinc-700/50 bg-zinc-800/50 hover:border-zinc-600"
              }`}
            >
              <h4 className="text-sm font-medium text-zinc-100">
                {HOST_DISPLAY_NAMES[host]}
              </h4>
            </button>
          ))}
        </div>
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
            {selectedHost &&
              ` Your library is now accessible from ${HOST_DISPLAY_NAMES[selectedHost]}.`}
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

      {currentStep === "install-homebrew" && isMacOS && (
        <StepCard
          title="Install Homebrew"
          description="Homebrew is required to install the CandleKeep CLI on macOS."
          actionLabel="Install Homebrew"
          onAction={handleInstallHomebrew}
          disabled={stepState.status === "running"}
        />
      )}

      {currentStep === "install-cli" && (
        <StepCard
          title="Install CandleKeep CLI"
          description={
            isMacOS
              ? "The CLI manages your library and handles authentication. Installing via Homebrew."
              : "The CLI manages your library and handles authentication. Downloading from GitHub."
          }
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

      {currentStep === "install-host" && selectedHost && (
        <StepCard
          title={`Install ${HOST_DISPLAY_NAMES[selectedHost]}`}
          description={`${HOST_DISPLAY_NAMES[selectedHost]} needs to be installed first. Install it, then come back.`}
          actionLabel={`I've Installed ${HOST_DISPLAY_NAMES[selectedHost]}`}
          onAction={async () => {
            if (!selectedHost) return;
            const status = await cmd.checkIntegration(selectedHost);
            if (status.host_installed) {
              if (!status.integration_installed) {
                setCurrentStep("install-integration");
              } else {
                setCurrentStep("done");
              }
            } else {
              setStepState({
                status: "error",
                message: `${HOST_DISPLAY_NAMES[selectedHost]} not detected. Please install it first.`,
              });
            }
          }}
          disabled={stepState.status === "running"}
        />
      )}

      {currentStep === "install-integration" && selectedHost && (
        <StepCard
          title={`Install CandleKeep for ${HOST_DISPLAY_NAMES[selectedHost]}`}
          description={`This connects your CandleKeep library to ${HOST_DISPLAY_NAMES[selectedHost]}.`}
          actionLabel="Install Integration"
          onAction={handleInstallIntegration}
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
