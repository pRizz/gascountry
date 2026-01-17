import { ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Repository } from "@/types/ralphtown";

interface RepoSelectorProps {
  repositories: Repository[];
  selectedRepo: Repository | null;
  selectedBranch: string;
  onSelectRepo: (repo: Repository) => void;
  onSelectBranch: (branch: string) => void;
}

export function RepoSelector({
  repositories,
  selectedRepo,
  selectedBranch,
  onSelectRepo,
  onSelectBranch,
}: RepoSelectorProps) {
  return (
    <div className="flex items-center gap-2 text-sm">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            className="h-auto py-1 px-2 text-muted-foreground hover:text-foreground gap-1"
          >
            {selectedRepo ? selectedRepo.fullName : "Select repository"}
            <ChevronDown className="h-3.5 w-3.5" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-64">
          {repositories.map((repo) => (
            <DropdownMenuItem
              key={repo.id}
              onClick={() => onSelectRepo(repo)}
              className="cursor-pointer"
            >
              {repo.fullName}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      {selectedRepo && (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              className="h-auto py-1 px-2 text-muted-foreground hover:text-foreground gap-1"
            >
              {selectedBranch}
              <ChevronDown className="h-3.5 w-3.5" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {selectedRepo.branches.map((branch) => (
              <DropdownMenuItem
                key={branch}
                onClick={() => onSelectBranch(branch)}
                className="cursor-pointer"
              >
                {branch}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      )}
    </div>
  );
}
