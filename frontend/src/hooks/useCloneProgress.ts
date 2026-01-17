import { useRef, useCallback, useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "@/api/hooks";
import type { CloneProgress, Repo } from "@/api/types";

export interface UseCloneProgressOptions {
  onProgress: (progress: CloneProgress) => void;
  onComplete: (repo: Repo, message: string) => void;
  onError: (message: string, helpSteps?: string[]) => void;
}

export interface UseCloneProgressReturn {
  startClone: (url: string) => void;
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

      // Handle error event
      eventSource.addEventListener("error", (event) => {
        // Check if it's a custom error event with data
        const messageEvent = event as MessageEvent;
        if (messageEvent.data) {
          try {
            const data = JSON.parse(messageEvent.data) as {
              message: string;
              help_steps?: string[];
            };
            onErrorRef.current(data.message, data.help_steps);
          } catch {
            onErrorRef.current("Clone failed");
          }
        } else {
          // Connection error
          onErrorRef.current("Connection to server lost");
        }
        eventSource.close();
        eventSourceRef.current = null;
      });

      eventSourceRef.current = eventSource;
    },
    [cancel, queryClient]
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      cancel();
    };
  }, [cancel]);

  return { startClone, cancel };
}
