import { useState } from "react";
import { Search, SlidersHorizontal, SquarePen } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AgentListItem } from "./AgentListItem";
import { GastownInstance } from "@/types/gastown";

interface AgentSidebarProps {
  instances: GastownInstance[];
  activeInstanceId: string | null;
  onSelectInstance: (id: string | null) => void;
  onNewGastown: () => void;
}

export function AgentSidebar({
  instances,
  activeInstanceId,
  onSelectInstance,
  onNewGastown,
}: AgentSidebarProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const filteredInstances = instances.filter(
    (instance) =>
      instance.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      instance.repo.toLowerCase().includes(searchQuery.toLowerCase())
  );

  // Group instances by time
  const groupInstances = (instances: GastownInstance[]) => {
    const now = Date.now();
    const yesterday: GastownInstance[] = [];
    const thisWeek: GastownInstance[] = [];
    const older: GastownInstance[] = [];

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
      {/* Logo */}
      <div className="p-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded bg-foreground flex items-center justify-center">
            <span className="text-background font-bold text-sm">G</span>
          </div>
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
            placeholder="Search agents..."
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

      {/* New Gascountry Button */}
      <div className="px-3 pb-3">
        <Button
          onClick={onNewGastown}
          variant="outline"
          className="w-full justify-center text-sm h-9 border-sidebar-border bg-sidebar-accent hover:bg-accent"
        >
          New gascountry
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
    </aside>
  );
}
