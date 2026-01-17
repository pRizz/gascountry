import { useState } from "react";
import { Search, SlidersHorizontal, SquarePen, Github, Globe } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AgentListItem } from "./AgentListItem";
import { SettingsDialog } from "./SettingsDialog";
import { RalphtownInstance } from "@/types/ralphtown";

interface AgentSidebarProps {
  instances: RalphtownInstance[];
  activeInstanceId: string | null;
  onSelectInstance: (id: string | null) => void;
  onNewSession: () => void;
}

export function AgentSidebar({
  instances,
  activeInstanceId,
  onSelectInstance,
  onNewSession,
}: AgentSidebarProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const filteredInstances = instances.filter(
    (instance) =>
      instance.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      instance.repo.toLowerCase().includes(searchQuery.toLowerCase())
  );

  // Group instances by time
  const groupInstances = (instances: RalphtownInstance[]) => {
    const now = Date.now();
    const yesterday: RalphtownInstance[] = [];
    const thisWeek: RalphtownInstance[] = [];
    const older: RalphtownInstance[] = [];

    instances.forEach((instance) => {
      const hoursAgo = (now - instance.createdAt.getTime()) / (1000 * 60 * 60);
      if (hoursAgo < 24) {
        yesterday.push(instance);
      } else if (hoursAgo < 7 * 24) {
        thisWeek.push(instance);
      } else {
        older.push(instance);
      }
    });

    return { yesterday, thisWeek, older };
  };

  const grouped = groupInstances(filteredInstances);

  return (
    <aside className="w-80 h-screen bg-sidebar border-r border-sidebar-border flex flex-col">
      {/* Logo and Title */}
      <div className="p-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded bg-foreground flex items-center justify-center">
            <Globe className="h-4 w-4 text-background" />
          </div>
          <span className="font-semibold text-foreground">Ralphtown</span>
        </div>
        <Button variant="ghost" size="icon" className="h-7 w-7 text-muted-foreground">
          <SquarePen className="h-4 w-4" />
        </Button>
      </div>

      {/* Search */}
      <div className="px-3 pb-2">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search sessions..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 pr-9 bg-sidebar-accent border-sidebar-border text-sm h-9"
          />
          <Button
            variant="ghost"
            size="icon"
            className="absolute right-1 top-1/2 -translate-y-1/2 h-7 w-7 text-muted-foreground"
          >
            <SlidersHorizontal className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {/* New Session Button */}
      <div className="px-3 pb-3">
        <Button
          onClick={onNewSession}
          variant="outline"
          className="w-full justify-center text-sm h-9 border-sidebar-border bg-sidebar-accent hover:bg-accent"
        >
          New session
        </Button>
      </div>

      {/* Agent List */}
      <div className="flex-1 overflow-y-auto px-2 pb-4">
        {grouped.yesterday.length > 0 && (
          <div className="mb-4">
            <h3 className="px-3 py-2 text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Yesterday
            </h3>
            <div className="space-y-0.5">
              {grouped.yesterday.map((instance) => (
                <AgentListItem
                  key={instance.id}
                  instance={instance}
                  isActive={activeInstanceId === instance.id}
                  onClick={() => onSelectInstance(instance.id)}
                />
              ))}
            </div>
          </div>
        )}

        {grouped.thisWeek.length > 0 && (
          <div className="mb-4">
            <h3 className="px-3 py-2 text-xs font-medium text-muted-foreground uppercase tracking-wider">
              This Week
            </h3>
            <div className="space-y-0.5">
              {grouped.thisWeek.map((instance) => (
                <AgentListItem
                  key={instance.id}
                  instance={instance}
                  isActive={activeInstanceId === instance.id}
                  onClick={() => onSelectInstance(instance.id)}
                />
              ))}
            </div>
          </div>
        )}

        {grouped.older.length > 0 && (
          <div>
            <h3 className="px-3 py-2 text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Older
            </h3>
            <div className="space-y-0.5">
              {grouped.older.map((instance) => (
                <AgentListItem
                  key={instance.id}
                  instance={instance}
                  isActive={activeInstanceId === instance.id}
                  onClick={() => onSelectInstance(instance.id)}
                />
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Footer with Settings and Source Code Link */}
      <div className="px-3 py-3 border-t border-sidebar-border mt-auto flex items-center justify-between">
        <a
          href="https://github.com/pRizz/ralphtown"
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          <Github className="h-4 w-4" />
          <span>ralphtown source code</span>
        </a>
        <SettingsDialog />
      </div>
    </aside>
  );
}
