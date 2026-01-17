import { useState, useEffect, useMemo, useRef } from "react";
import { User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { RepoSelector } from "./RepoSelector";
import { PromptInput } from "./PromptInput";
import { ConversationView } from "./ConversationView";
import { RalphtownInstance, Repository, mapApiRepoToRepository } from "@/types/ralphtown";
import type { Repo } from "@/api/types";
import type { OutputLine } from "@/hooks/useWebSocket";

interface MainPanelProps {
  activeInstance: RalphtownInstance | null;
  onStartSession: (prompt: string, repo: Repository, branch: string, model: string) => void;
  onSendMessage: (instanceId: string, message: string) => void;
  onCancel?: (instanceId: string) => void;
  repos: Repo[];
  outputLines?: OutputLine[];
}

export function MainPanel({ activeInstance, onStartSession, onSendMessage, onCancel, repos, outputLines = [] }: MainPanelProps) {
  // Convert API repos to UI repositories
  const repositories = useMemo(() => {
    return repos.map((repo) => mapApiRepoToRepository(repo));
  }, [repos]);

  const [selectedRepo, setSelectedRepo] = useState<Repository | null>(null);
  const [selectedBranch, setSelectedBranch] = useState("main");

  // Track a pending repo selection (by ID) that should be applied when repos list updates
  const pendingRepoIdRef = useRef<string | null>(null);

  // Initialize or reconcile selected repo when repo list changes
  useEffect(() => {
    if (repositories.length === 0) {
      if (selectedRepo) {
        setSelectedRepo(null);
      }
      return;
    }

    // Check if there's a pending repo selection to apply
    if (pendingRepoIdRef.current) {
      const pendingRepo = repositories.find((repo) => repo.id === pendingRepoIdRef.current);
      if (pendingRepo) {
        setSelectedRepo(pendingRepo);
        setSelectedBranch(pendingRepo.defaultBranch);
        pendingRepoIdRef.current = null;
        return;
      }
      // Pending repo not found yet, keep waiting (don't clear the ref)
    }

    if (!selectedRepo) {
      setSelectedRepo(repositories[0]);
      setSelectedBranch(repositories[0].defaultBranch);
      return;
    }

    const matchedRepo = repositories.find((repo) => repo.id === selectedRepo.id);
    if (!matchedRepo) {
      // Only reset to first repo if there's no pending selection
      if (!pendingRepoIdRef.current) {
        setSelectedRepo(repositories[0]);
        setSelectedBranch(repositories[0].defaultBranch);
      }
      return;
    }

    if (matchedRepo !== selectedRepo) {
      setSelectedRepo(matchedRepo);
      if (!matchedRepo.branches.includes(selectedBranch)) {
        setSelectedBranch(matchedRepo.defaultBranch);
      }
    }
  }, [repositories, selectedRepo, selectedBranch]);

  const handleSelectRepo = (repo: Repository) => {
    // Store the repo ID as pending in case the repos list hasn't updated yet
    pendingRepoIdRef.current = repo.id;
    setSelectedRepo(repo);
    setSelectedBranch(repo.defaultBranch);
  };

  const handleSubmit = (prompt: string, model: string) => {
    if (selectedRepo) {
      onStartSession(prompt, selectedRepo, selectedBranch, model);
    }
  };

  return (
    <div className="flex-1 flex flex-col bg-background h-screen">
      {/* Header */}
      <header className="flex items-center justify-end px-6 py-3 border-b border-border flex-shrink-0">
        <div className="flex items-center gap-3">
          <Button variant="ghost" className="text-sm text-muted-foreground hover:text-foreground">
            Dashboard
          </Button>
          <Button
            variant="outline"
            size="icon"
            className="h-8 w-8 rounded-full border-border"
          >
            <User className="h-4 w-4" />
          </Button>
        </div>
      </header>

      {/* Main Content */}
      {activeInstance ? (
        <ConversationView
          instance={activeInstance}
          onSendMessage={onSendMessage}
          onCancel={onCancel}
          outputLines={outputLines}
        />
      ) : (
        <main className="flex-1 flex flex-col items-center justify-center px-6">
          <div className="w-full max-w-2xl flex flex-col items-center gap-6">
            {/* Repo Selector */}
            <RepoSelector
              repositories={repositories}
              selectedRepo={selectedRepo}
              selectedBranch={selectedBranch}
              onSelectRepo={handleSelectRepo}
              onSelectBranch={setSelectedBranch}
            />

            {/* Prompt Input */}
            <PromptInput onSubmit={handleSubmit} />
          </div>
        </main>
      )}
    </div>
  );
}
