import { useState } from "react";
import { ChevronDown, Plus, GitBranch } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAddRepo } from "@/api/hooks";
import { useToast } from "@/hooks/use-toast";
import { cn } from "@/lib/utils";
import { Repository, mapApiRepoToRepository } from "@/types/ralphtown";
import { CloneDialog } from "./CloneDialog";
import type { Repo } from "@/api/types";

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
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [isCloneOpen, setIsCloneOpen] = useState(false);
  const [isDragActive, setIsDragActive] = useState(false);
  const [customPath, setCustomPath] = useState("");
  const addRepo = useAddRepo();
  const { toast } = useToast();

  const handleCloneSuccess = (repo: Repo) => {
    setIsDropdownOpen(false);
    onSelectRepo(mapApiRepoToRepository(repo));
  };

  const handleAddRepo = async () => {
    const trimmedPath = customPath.trim();
    if (!trimmedPath) {
      toast({
        title: "Path required",
        description: "Enter a local path to add a repository.",
        variant: "destructive",
      });
      return;
    }

    try {
      const repo = await addRepo.mutateAsync({ path: trimmedPath });
      onSelectRepo(mapApiRepoToRepository(repo));
      setCustomPath("");
      setIsAddOpen(false);
      toast({
        title: "Repository added",
        description: repo.name,
      });
    } catch (error) {
      toast({
        title: "Failed to add repository",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    }
  };

  const handleDropPath = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setIsDragActive(false);

    const item = event.dataTransfer.items?.[0];
    const file = item?.kind === "file" ? item.getAsFile() : null;
    if (!file) {
      return;
    }

    const maybeFilePath = (file as File & { path?: string }).path;
    if (maybeFilePath) {
      setCustomPath(maybeFilePath);
      return;
    }

    toast({
      title: "Path unavailable",
      description: "Your browser does not expose full paths. Paste the path instead.",
      variant: "destructive",
    });
  };

  return (
    <div className="flex items-center gap-2 text-sm">
      <Dialog
        open={isAddOpen}
        onOpenChange={(open) => {
          setIsAddOpen(open);
          if (!open) setCustomPath("");
        }}
      >
        <DropdownMenu open={isDropdownOpen} onOpenChange={setIsDropdownOpen}>
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
            {repositories.length > 0 ? (
              repositories.map((repo) => (
                <DropdownMenuItem
                  key={repo.id}
                  onClick={() => onSelectRepo(repo)}
                  className="cursor-pointer"
                >
                  {repo.fullName}
                </DropdownMenuItem>
              ))
            ) : (
              <DropdownMenuItem disabled className="text-muted-foreground">
                No repositories found
              </DropdownMenuItem>
            )}
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onSelect={(event) => {
                event.preventDefault();
                setIsAddOpen(true);
              }}
              className="cursor-pointer text-foreground"
            >
              <Plus className="mr-2 h-3.5 w-3.5" />
              Add local path...
            </DropdownMenuItem>
            <DropdownMenuItem
              onSelect={(event) => {
                event.preventDefault();
                setIsCloneOpen(true);
              }}
              className="cursor-pointer text-foreground"
            >
              <GitBranch className="mr-2 h-3.5 w-3.5" />
              Clone from URL...
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <DialogContent className="sm:max-w-[480px]">
          <DialogHeader>
            <DialogTitle>Add local repository</DialogTitle>
            <DialogDescription>
              Enter the full path to a local git repository.
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="repoPath" className="text-right">
                Path
              </Label>
              <div
                className={cn(
                  "col-span-3 rounded-md border border-dashed border-muted-foreground/40 p-2 transition-colors",
                  isDragActive && "border-primary bg-accent/30"
                )}
                onDrop={handleDropPath}
                onDragOver={(event) => {
                  event.preventDefault();
                  setIsDragActive(true);
                }}
                onDragLeave={() => setIsDragActive(false)}
              >
                <Input
                  id="repoPath"
                  value={customPath}
                  onChange={(event) => setCustomPath(event.target.value)}
                  placeholder="~/Projects/my-repo"
                  className="border-0 bg-transparent px-0 focus-visible:ring-0"
                />
                <p className="mt-2 text-xs text-muted-foreground">
                  Drag &amp; drop a folder here, or paste a full path.
                </p>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setIsAddOpen(false)}
              disabled={addRepo.isPending}
            >
              Cancel
            </Button>
            <Button onClick={handleAddRepo} disabled={addRepo.isPending}>
              {addRepo.isPending ? "Adding..." : "Add repository"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <CloneDialog
        open={isCloneOpen}
        onOpenChange={setIsCloneOpen}
        onCloneSuccess={handleCloneSuccess}
      />

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
