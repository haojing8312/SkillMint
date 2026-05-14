type TauriBrowserWindow = Window & {
  __TAURI_INTERNALS__?: {
    invoke?: unknown;
  } | unknown;
  __WORKCLAW_FORCE_BROWSER_ONLY__?: boolean;
};

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof (error as { message?: unknown }).message === "string"
  ) {
    return (error as { message: string }).message;
  }
  return "";
}

export function isTauriRuntimeAvailable(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  const browserWindow = window as TauriBrowserWindow;
  if (browserWindow.__WORKCLAW_FORCE_BROWSER_ONLY__) {
    return false;
  }
  if (import.meta.env.MODE === "test") {
    return true;
  }
  const internals = browserWindow.__TAURI_INTERNALS__;
  return (
    typeof internals === "object" &&
    internals !== null &&
    typeof (internals as { invoke?: unknown }).invoke === "function"
  );
}

export function isTauriInvokeUnavailableError(error: unknown): boolean {
  const message = getErrorMessage(error);

  return (
    message.includes("Cannot read properties of undefined (reading 'invoke')") ||
    message.includes("window.__TAURI_INTERNALS__") ||
    message.includes("__TAURI_INTERNALS__")
  );
}
