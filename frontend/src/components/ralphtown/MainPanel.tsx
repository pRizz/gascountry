import { useState, useEffect, useMemo } from "react";
import { User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { RepoSelector } from "./RepoSelector";
import { PromptInput } from "./PromptInput";
import { ConversationView } from "./ConversationView";
import { RalphtownInstance, Repository, mapApiRepoToRepository } from "@/types/ralphtown";
import type { Repo } from "@/api/types";

interface MainPanelProps {
  activeInstance: RalphtownInstance | null;
  onStartSession: (prompt: string, repo: Repository, branch: string, model: string) => void;
  onSendMessage: (instanceId: string, message: string) => void;
  repos: Repo[];
}

export function MainPanel({ activeInstance, onStartSession, onSendMessage, repos }: MainPanelProps) {
  // Convert API repos to UI repositories
  const repositories = useMemo(() => {
    return repos.map((repo) => mapApiRepoToRepository(repo));
  }, [repos]);

  const [selectedRepo, setSelectedRepo] = useState<Repository | null>(null);
  const [selectedBranch, setSelectedBranch] = useState("main");

  // Initialize selected repo when repos load
  useEffect(() => {
    if (repositories.length > 0 && !selectedRepo) {
      setSelectedRepo(repositories[0]);
      setSelectedBranch(repositories[0].defaultBranch);
    }
  }, [repositories, selectedRepo]);

  const handleSelectRepo = (repo: Repository) => {
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
        <ConversationView instance={activeInstance} onSendMessage={onSendMessage} />
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
