import { fireEvent, render, screen, waitFor } from "@testing-library/react";

import { ChatView } from "../ChatView";

const invokeMock = vi.fn<(command: string, payload?: unknown) => Promise<unknown>>();

let messagesResponse: any[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, payload?: unknown) => invokeMock(command, payload),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

describe("ChatView virtualization", () => {
  beforeEach(() => {
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: vi.fn(),
    });
    Object.defineProperty(HTMLElement.prototype, "scrollTo", {
      configurable: true,
      value: vi.fn(),
    });
    messagesResponse = [];
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === "get_messages") return Promise.resolve(messagesResponse);
      if (command === "list_sessions") return Promise.resolve([]);
      if (command === "get_sessions") return Promise.resolve([]);
      if (command === "list_session_runs") return Promise.resolve([]);
      return Promise.resolve(null);
    });
  });

  test("renders only a bounded slice of a long conversation after scroll metrics are known", async () => {
    messagesResponse = Array.from({ length: 200 }, (_, index) => ({
      id: `message-${index}`,
      role: "user",
      content: `历史消息 ${index + 1}`,
      created_at: new Date(2026, 3, 1, 9, 0, index).toISOString(),
    }));

    const { container } = render(
      <ChatView
        skill={{
          id: "builtin-general",
          name: "General",
          description: "desc",
          version: "1.0.0",
          author: "test",
          recommended_model: "",
          tags: [],
          created_at: new Date().toISOString(),
        }}
        models={[
          {
            id: "m1",
            name: "model",
            api_format: "openai",
            base_url: "https://example.com",
            model_name: "model",
            is_default: true,
          },
        ]}
        sessionId="sess-virtualized"
      />,
    );

    const scrollRegion = await screen.findByTestId("chat-scroll-region");

    Object.defineProperty(scrollRegion, "scrollTop", {
      configurable: true,
      value: 3200,
      writable: true,
    });
    Object.defineProperty(scrollRegion, "clientHeight", {
      configurable: true,
      value: 640,
    });
    Object.defineProperty(scrollRegion, "scrollHeight", {
      configurable: true,
      value: 18400,
    });

    fireEvent.scroll(scrollRegion);

    await waitFor(() => {
      expect(screen.queryByTestId("chat-message-0")).not.toBeInTheDocument();
    });

    expect(screen.getByTestId("chat-message-34")).toBeInTheDocument();
    expect(screen.queryByTestId("chat-message-199")).not.toBeInTheDocument();

    const mountedMessages = container.querySelectorAll(
      "[data-testid^='chat-message-']:not([data-testid^='chat-message-bubble-'])",
    );
    expect(mountedMessages.length).toBeLessThan(40);
  });
});
