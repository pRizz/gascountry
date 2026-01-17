import { useState, useMemo } from "react";
import { AgentSidebar } from "@/components/ralphtown/AgentSidebar";
import { MainPanel } from "@/components/ralphtown/MainPanel";
import {
  RalphtownInstance,
  Repository,
  ChatMessage,
  mapApiSessionToInstance,
} from "@/types/ralphtown";
import { useToast } from "@/hooks/use-toast";
import {
  useSessions,
  useSession,
  useRepos,
  useCreateSession,
  useRunSession,
} from "@/api/hooks";
import type { Repo } from "@/api/types";

const Index = () => {
  const [activeInstanceId, setActiveInstanceId] = useState<string | null>(null);
  const { toast } = useToast();

  // Fetch sessions and repos from API
  const { data: sessions = [], isLoading: sessionsLoading } = useSessions();
  const { data: repos = [], isLoading: reposLoading } = useRepos();
  const { data: activeSessionDetails } = useSession(activeInstanceId);

  // Mutations
  const createSession = useCreateSession();
  const runSession = useRunSession();

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
    _model: string
  ) => {
    try {
      // Create session
      const session = await createSession.mutateAsync({
        repo_id: repo.id,
        name: prompt.length > 30 ? prompt.slice(0, 30) + "..." : prompt,
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
    // For now, follow-up messages just run ralph again
    // TODO: Implement proper message handling through WebSocket
    runSession.mutate({
      id: instanceId,
      req: { prompt: content },
    });
  };

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
        repos={repos}
      />
    </div>
  );
};

export default Index;
