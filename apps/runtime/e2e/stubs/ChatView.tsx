type ChatViewProps = {
  sessionId: string;
  initialMessage?: string;
};

export function ChatView(props: ChatViewProps): JSX.Element {
  return (
    <div data-testid="e2e-chat-view" className="h-full p-4">
      <div>chat-view-stub</div>
      <div data-testid="e2e-chat-session-id">{props.sessionId}</div>
      {props.initialMessage ? (
        <div data-testid="e2e-chat-initial-message">{props.initialMessage}</div>
      ) : null}
    </div>
  );
}
