import { useState } from "react";
import { Button } from "@/components/ui/button";
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
import { useCloneRepo } from "@/api/hooks";
import { useToast } from "@/hooks/use-toast";
import type { Repo } from "@/api/types";

interface CloneDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCloneSuccess: (repo: Repo) => void;
}

export function CloneDialog({ open, onOpenChange, onCloneSuccess }: CloneDialogProps) {
  const [gitUrl, setGitUrl] = useState("");
  const cloneRepo = useCloneRepo();
  const { toast } = useToast();

  const handleClone = async () => {
    const trimmedUrl = gitUrl.trim();
    if (!trimmedUrl) {
      toast({
        title: "URL required",
        description: "Enter a git URL to clone.",
        variant: "destructive",
      });
      return;
    }

    try {
      const response = await cloneRepo.mutateAsync({ url: trimmedUrl });
      onCloneSuccess(response.repo);
      setGitUrl("");
      onOpenChange(false);
      toast({
        title: "Repository cloned",
        description: response.message,
      });
    } catch (error) {
      toast({
        title: "Failed to clone repository",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    }
  };

  const handleOpenChange = (newOpen: boolean) => {
    onOpenChange(newOpen);
    if (!newOpen) {
      setGitUrl("");
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[480px]">
        <DialogHeader>
          <DialogTitle>Clone from URL</DialogTitle>
          <DialogDescription>
            Enter a git URL (SSH or HTTPS) to clone the repository.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="gitUrl" className="text-right">
              URL
            </Label>
            <Input
              id="gitUrl"
              value={gitUrl}
              onChange={(e) => setGitUrl(e.target.value)}
              placeholder="https://github.com/user/repo.git"
              className="col-span-3"
              onKeyDown={(e) => {
                if (e.key === "Enter" && !cloneRepo.isPending) {
                  handleClone();
                }
              }}
            />
          </div>
          <p className="text-xs text-muted-foreground ml-auto col-span-3 pr-1">
            Repository will be cloned to ~/ralphtown/
          </p>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => handleOpenChange(false)}
            disabled={cloneRepo.isPending}
          >
            Cancel
          </Button>
          <Button onClick={handleClone} disabled={cloneRepo.isPending}>
            {cloneRepo.isPending ? "Cloning..." : "Clone"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
