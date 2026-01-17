import { RalphtownInstance } from "@/types/ralphtown";
import { Check, Loader2, AlertCircle, Clock } from "lucide-react";
import { cn } from "@/lib/utils";

interface AgentListItemProps {
  instance: RalphtownInstance;
  isActive: boolean;
  onClick: () => void;
}

const statusIcons = {
  completed: Check,
  running: Loader2,
  error: AlertCircle,
  pending: Clock,
  idle: Clock,
  cancelled: AlertCircle,
};

export function AgentListItem({ instance, isActive, onClick }: AgentListItemProps) {
  const StatusIcon = statusIcons[instance.status];

  const formatTimeAgo = (date: Date) => {
    const hours = Math.floor((Date.now() - date.getTime()) / (1000 * 60 * 60));
    if (hours < 24) return `${hours}h`;
    const days = Math.floor(hours / 24);
    return `${days}d`;
  };

  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full text-left px-3 py-2.5 rounded-md transition-colors group",
        "hover:bg-sidebar-accent",
        isActive && "bg-sidebar-accent"
      )}
    >
      <div className="flex items-start gap-2.5">
        <StatusIcon
          className={cn(
            "h-4 w-4 mt-0.5 flex-shrink-0",
            instance.status === "completed" && "text-muted-foreground",
            instance.status === "running" && "text-agent-running animate-spin",
            instance.status === "error" && "text-agent-error",
            instance.status === "pending" && "text-agent-pending",
            instance.status === "idle" && "text-muted-foreground",
            instance.status === "cancelled" && "text-agent-error"
          )}
        />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between gap-2">
            <span className="text-sm font-medium text-foreground truncate">
              {instance.title}
            </span>
            <span className="text-xs text-muted-foreground flex-shrink-0">
              {formatTimeAgo(instance.createdAt)}
            </span>
          </div>
          <div className="flex items-center gap-1.5 mt-0.5">
            <span className="text-xs text-muted-foreground">{instance.repo}</span>
          </div>
        </div>
      </div>
    </button>
  );
}
