import { useState } from "react";
import { User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { RepoSelector } from "./RepoSelector";
import { PromptInput } from "./PromptInput";
import { ConversationView } from "./ConversationView";
import { RalphtownInstance, Repository } from "@/types/ralphtown";
import { mockRepositories } from "@/data/mockData";

interface MainPanelProps {
  activeInstance: RalphtownInstance | null;
  onStartSession: (prompt: string, repo: Repository, branch: string, model: string) => void;
  onSendMessage: (instanceId: string, message: string) => void;
}

export function MainPanel({ activeInstance, onStartSession, onSendMessage }: MainPanelProps) {
  const [selectedRepo, setSelectedRepo] = useState<Repository | null>(mockRepositories[0]);
  const [selectedBranch, setSelectedBranch] = useState(mockRepositories[0].defaultBranch);

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
              repositories={mockRepositories}
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
