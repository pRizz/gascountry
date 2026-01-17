import { useRef, useCallback, useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "@/api/hooks";
import type { CloneProgress, Repo, CredentialRequest, AuthType } from "@/api/types";

export interface UseCloneProgressOptions {
  onProgress: (progress: CloneProgress) => void;
  onComplete: (repo: Repo, message: string) => void;
  onError: (message: string, helpSteps?: string[], authType?: AuthType, canRetry?: boolean) => void;
}

export interface UseCloneProgressReturn {
  startClone: (url: string) => void;
  startCloneWithCredentials: (url: string, credentials: CredentialRequest) => void;
  cancel: () => void;
}

export function useCloneProgress(
  options: UseCloneProgressOptions
): UseCloneProgressReturn {
  const eventSourceRef = useRef<EventSource | null>(null);
  const queryClient = useQueryClient();

  // Store callbacks in refs to avoid stale closures
  const onProgressRef = useRef(options.onProgress);
  const onCompleteRef = useRef(options.onComplete);
  const onErrorRef = useRef(options.onError);

  useEffect(() => {
    onProgressRef.current = options.onProgress;
    onCompleteRef.current = options.onComplete;
    onErrorRef.current = options.onError;
  }, [options.onProgress, options.onComplete, options.onError]);

  const cancel = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }
  }, []);

  const startClone = useCallback(
    (url: string) => {
      // Close any existing EventSource
      cancel();

      // Create new EventSource connection
      const encodedUrl = encodeURIComponent(url);
      const eventSource = new EventSource(
        `/api/repos/clone-progress?url=${encodedUrl}`
      );

      // Handle progress messages (default message event)
      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as CloneProgress;
          onProgressRef.current(data);
        } catch (e) {
          console.error("Failed to parse progress message:", e);
        }
      };

      // Handle complete event
      eventSource.addEventListener("complete", (event) => {
        try {
          const messageEvent = event as MessageEvent;
          const data = JSON.parse(messageEvent.data) as {
            repo: Repo;
            message: string;
          };
          // Invalidate repos query to refresh the list
          queryClient.invalidateQueries({ queryKey: queryKeys.repos });
          onCompleteRef.current(data.repo, data.message);
          eventSource.close();
          eventSourceRef.current = null;
        } catch (e) {
          console.error("Failed to parse complete message:", e);
        }
      });

      // Track if we received a server error to avoid duplicate error handling
      let receivedServerError = false;

      // Handle clone_error event (custom event from server with error details)
      eventSource.addEventListener("clone_error", (event) => {
        receivedServerError = true;
        const messageEvent = event as MessageEvent;
        try {
          const data = JSON.parse(messageEvent.data) as {
            message: string;
            help_steps?: string[];
            auth_type?: AuthType;
            can_retry_with_credentials?: boolean;
          };
          onErrorRef.current(data.message, data.help_steps, data.auth_type, data.can_retry_with_credentials);
        } catch {
          onErrorRef.current("Clone failed");
        }
        eventSource.close();
        eventSourceRef.current = null;
      });

      // Handle connection-level errors (browser's built-in error event)
      eventSource.addEventListener("error", () => {
        // Only report connection error if we didn't already receive a server error
        if (!receivedServerError) {
          onErrorRef.current("Connection to the server was lost");
        }
        eventSource.close();
        eventSourceRef.current = null;
      });

      eventSourceRef.current = eventSource;
    },
    [cancel, queryClient]
  );

  // Start clone with credentials via POST (for retry after auth failure)
  const startCloneWithCredentials = useCallback(
    async (url: string, credentials: CredentialRequest) => {
      cancel();

      try {
        const response = await fetch("/api/repos/clone-progress", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url, credentials }),
        });

        if (!response.ok || !response.body) {
          onErrorRef.current("Failed to start clone");
          return;
        }

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split("\n\n");
          buffer = lines.pop() || "";

          for (const chunk of lines) {
            const dataMatch = chunk.match(/^data: (.+)$/m);
            if (dataMatch) {
              try {
                const data = JSON.parse(dataMatch[1]);
                if (data.type === "progress") {
                  onProgressRef.current(data.data);
                } else if (data.type === "complete") {
                  queryClient.invalidateQueries({ queryKey: queryKeys.repos });
                  onCompleteRef.current(data.data.repo, data.data.message);
                  return;
                } else if (data.type === "error") {
                  onErrorRef.current(
                    data.data.message,
                    data.data.help_steps,
                    data.data.auth_type,
                    data.data.can_retry_with_credentials
                  );
                  return;
                }
              } catch (e) {
                console.error("Failed to parse SSE message:", e);
              }
            }
          }
        }
      } catch {
        onErrorRef.current("Connection error");
      }
    },
    [cancel, queryClient]
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      cancel();
    };
  }, [cancel]);

  return { startClone, startCloneWithCredentials, cancel };
}
