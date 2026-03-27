import "@testing-library/jest-dom";
import { cleanup } from "@testing-library/react";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.restoreAllMocks();
  vi.useRealTimers();
  window.localStorage.clear();
  window.sessionStorage.clear();
  window.location.hash = "";
});
