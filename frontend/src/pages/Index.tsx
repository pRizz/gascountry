import { useState, useMemo, useCallback, useEffect } from "react";
import { AgentSidebar } from "@/components/ralphtown/AgentSidebar";
import { MainPanel } from "@/components/ralphtown/MainPanel";
import {
  RalphtownInstance,
  Repository,
  ChatMessage,
  mapApiSessionToInstance,
} from "@/types/ralphtown";
import { useToast } from "@/hooks/use-toast";
import { useWebSocket, OutputLine } from "@/hooks/useWebSocket";
import {
  useSessions,
  useSession,
  useRepos,
  useCreateSession,
  useRunSession,
} from "@/api/hooks";
import { useQueryClient } from "@tanstack/react-query";
import type { Repo, SessionStatus, OrchestratorType } from "@/api/types";

const Index = () => {
  const [activeInstanceId, setActiveInstanceId] = useState<string | null>(null);
  const [outputLines, setOutputLines] = useState<Map<string, OutputLine[]>>(new Map());
  const [selectedOrchestrator, setSelectedOrchestrator] = useState<OrchestratorType>("ralph");
  const { toast } = useToast();
  const queryClient = useQueryClient();

  // Fetch sessions and repos from API
  const { data: sessions = [], isLoading: sessionsLoading } = useSessions();
  const { data: repos = [], isLoading: reposLoading } = useRepos();
  const { data: activeSessionDetails } = useSession(activeInstanceId);

  // Mutations
  const createSession = useCreateSession();
  const runSession = useRunSession();

  // WebSocket callbacks
  const handleWsOutput = useCallback((sessionId: string, line: OutputLine) => {
    setOutputLines((prev) => {
      const newMap = new Map(prev);
      const lines = newMap.get(sessionId) || [];
      newMap.set(sessionId, [...lines, line]);
      return newMap;
    });
  }, []);

  const handleWsStatus = useCallback(
    (sessionId: string, status: SessionStatus) => {
      // Invalidate queries to refetch session data
      queryClient.invalidateQueries({ queryKey: ["sessions"] });
      queryClient.invalidateQueries({ queryKey: ["session", sessionId] });

      // Clear output when session completes or errors
      if (status === "completed" || status === "error" || status === "cancelled") {
        // Optionally clear output after a delay to let user see final output
        setTimeout(() => {
          setOutputLines((prev) => {
            const newMap = new Map(prev);
            newMap.delete(sessionId);
            return newMap;
          });
        }, 5000);
      }
    },
    [queryClient]
  );

  const handleWsError = useCallback(
    (message: string) => {
      toast({
        title: "WebSocket Error",
        description: message,
        variant: "destructive",
      });
    },
    [toast]
  );

  // WebSocket hook
  const { isConnected, subscribe, unsubscribe, cancel } = useWebSocket({
    onOutput: handleWsOutput,
    onStatus: handleWsStatus,
    onError: handleWsError,
  });

  // Subscribe to active session's output
  useEffect(() => {
    if (activeInstanceId) {
      subscribe(activeInstanceId);
      return () => {
        unsubscribe(activeInstanceId);
      };
    }
  }, [activeInstanceId, subscribe, unsubscribe]);

  // Create a map of repos for quick lookup
  const repoMap = useMemo(() => {
    const map = new Map<string, Repo>();
    repos.forEach((repo) => map.set(repo.id, repo));
    return map;
  }, [repos]);

  // Convert API sessions to UI instances
  const instances: RalphtownInstance[] = useMemo(() => {
    return sessions.map((session) =>
      mapApiSessionToInstance(session, repoMap.get(session.repo_id))
    );
  }, [sessions, repoMap]);

  // Get full active instance with messages
  const activeInstance = useMemo(() => {
    if (!activeInstanceId || !activeSessionDetails) return null;
    return mapApiSessionToInstance(
      activeSessionDetails,
      repoMap.get(activeSessionDetails.repo_id)
    );
  }, [activeInstanceId, activeSessionDetails, repoMap]);

  const handleNewSession = () => {
    setActiveInstanceId(null);
  };

  const handleStartSession = async (
    prompt: string,
    repo: Repository,
    branch: string,
    _model: string,
    orchestrator: OrchestratorType
  ) => {
    try {
      // Create session
      const session = await createSession.mutateAsync({
        repo_id: repo.id,
        name: prompt.length > 30 ? prompt.slice(0, 30) + "..." : prompt,
        orchestrator,
      });

      // Start ralph with the prompt
      await runSession.mutateAsync({
        id: session.id,
        req: { prompt },
      });

      setActiveInstanceId(session.id);

      toast({
        title: "Session started",
        description: `Running on ${repo.name}`,
      });
    } catch (error) {
      toast({
        title: "Failed to start session",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    }
  };

  const handleSendMessage = (instanceId: string, content: string) => {
    // Clear previous output for this session
    setOutputLines((prev) => {
      const newMap = new Map(prev);
      newMap.delete(instanceId);
      return newMap;
    });

    // Run ralph with the new message
    runSession.mutate({
      id: instanceId,
      req: { prompt: content },
    });
  };

  const handleCancel = useCallback(
    (instanceId: string) => {
      cancel(instanceId);
      toast({
        title: "Cancelling session",
        description: "Sending cancel signal...",
      });
    },
    [cancel, toast]
  );

  // Get output lines for active session
  const activeOutputLines = activeInstanceId ? outputLines.get(activeInstanceId) || [] : [];

  const isLoading = sessionsLoading || reposLoading;

  if (isLoading) {
    return (
      <div className="flex min-h-screen w-full items-center justify-center">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen w-full">
      <AgentSidebar
        instances={instances}
        activeInstanceId={activeInstanceId}
        onSelectInstance={setActiveInstanceId}
        onNewSession={handleNewSession}
      />
      <MainPanel
        activeInstance={activeInstance}
        onStartSession={handleStartSession}
        onSendMessage={handleSendMessage}
        onCancel={handleCancel}
        repos={repos}
        outputLines={activeOutputLines}
        selectedOrchestrator={selectedOrchestrator}
        onSelectOrchestrator={setSelectedOrchestrator}
      />
    </div>
  );
};

export default Index;
