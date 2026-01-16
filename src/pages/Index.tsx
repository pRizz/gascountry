import { useState } from "react";
import { AgentSidebar } from "@/components/gastown/AgentSidebar";
import { MainPanel } from "@/components/gastown/MainPanel";
import { mockGastownInstances } from "@/data/mockData";
import { GastownInstance, Repository, ChatMessage } from "@/types/gastown";
import { useToast } from "@/hooks/use-toast";

const Index = () => {
  const [instances, setInstances] = useState<GastownInstance[]>(mockGastownInstances);
  const [activeInstanceId, setActiveInstanceId] = useState<string | null>(null);
  const { toast } = useToast();

  const activeInstance = activeInstanceId
    ? instances.find((i) => i.id === activeInstanceId) || null
    : null;

  const handleNewGastown = () => {
    setActiveInstanceId(null);
  };

  const handleSpawnGastown = (
    prompt: string,
    repo: Repository,
    branch: string,
    model: string
  ) => {
    const newInstance: GastownInstance = {
      id: crypto.randomUUID(),
      title: prompt.length > 30 ? prompt.slice(0, 30) + "..." : prompt,
      repo: repo.name,
      branch,
      status: "running",
      createdAt: new Date(),
      model,
      messages: [
        {
          id: crypto.randomUUID(),
          role: "user",
          content: prompt,
          timestamp: new Date(),
        },
        {
          id: crypto.randomUUID(),
          role: "agent",
          content: `Starting work on "${prompt}"...\n\nI'm analyzing the codebase in ${repo.fullName} on branch ${branch}.`,
          timestamp: new Date(),
        },
      ],
    };

    setInstances((prev) => [newInstance, ...prev]);
    setActiveInstanceId(newInstance.id);

    toast({
      title: "Gastown spawned",
      description: `Running "${newInstance.title}" on ${repo.fullName}`,
    });
  };

  const handleSendMessage = (instanceId: string, content: string) => {
    const userMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content,
      timestamp: new Date(),
    };

    const agentResponse: ChatMessage = {
      id: crypto.randomUUID(),
      role: "agent",
      content: `Understood. I'll work on that now...\n\nProcessing your request: "${content}"`,
      timestamp: new Date(),
    };

    setInstances((prev) =>
      prev.map((instance) =>
        instance.id === instanceId
          ? {
              ...instance,
              messages: [...instance.messages, userMessage, agentResponse],
            }
          : instance
      )
    );
  };

  return (
    <div className="flex min-h-screen w-full">
      <AgentSidebar
        instances={instances}
        activeInstanceId={activeInstanceId}
        onSelectInstance={setActiveInstanceId}
        onNewGastown={handleNewGastown}
      />
      <MainPanel
        activeInstance={activeInstance}
        onSpawnGastown={handleSpawnGastown}
        onSendMessage={handleSendMessage}
      />
    </div>
  );
};

export default Index;
