import { Bot, Sparkles, Users } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import type { OrchestratorType } from "@/api/types";

interface OrchestratorInfo {
  id: OrchestratorType;
  name: string;
  description: string;
  available: boolean;
  icon: React.ReactNode;
}

const ORCHESTRATORS: OrchestratorInfo[] = [
  {
    id: "ralph",
    name: "Ralph",
    description: "Autonomous coding agent",
    available: true,
    icon: <Bot className="h-4 w-4" />,
  },
  {
    id: "gsd",
    name: "GSD",
    description: "Task-driven orchestrator",
    available: false,
    icon: <Sparkles className="h-4 w-4" />,
  },
  {
    id: "gastown",
    name: "Gastown",
    description: "Multi-agent coordination",
    available: false,
    icon: <Users className="h-4 w-4" />,
  },
];

interface OrchestratorSelectorProps {
  value: OrchestratorType;
  onChange: (value: OrchestratorType) => void;
}

export function OrchestratorSelector({ value, onChange }: OrchestratorSelectorProps) {
  const selectedOrchestrator = ORCHESTRATORS.find((o) => o.id === value);

  return (
    <Select value={value} onValueChange={(v) => onChange(v as OrchestratorType)}>
      <SelectTrigger className="w-[180px]">
        <SelectValue>
          {selectedOrchestrator && (
            <span className="flex items-center gap-2">
              {selectedOrchestrator.icon}
              {selectedOrchestrator.name}
            </span>
          )}
        </SelectValue>
      </SelectTrigger>
      <SelectContent>
        {ORCHESTRATORS.map((orchestrator) => (
          <SelectItem
            key={orchestrator.id}
            value={orchestrator.id}
            disabled={!orchestrator.available}
          >
            <span className="flex items-center gap-2">
              {orchestrator.icon}
              <span>{orchestrator.name}</span>
              {!orchestrator.available && (
                <Badge variant="secondary" className="ml-1 text-[10px] px-1.5 py-0">
                  Coming Soon
                </Badge>
              )}
            </span>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
