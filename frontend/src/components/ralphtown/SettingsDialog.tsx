import { useState, useEffect } from "react";
import { Settings } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useConfig, useUpdateConfig, useBackends, usePresets } from "@/api/hooks";
import { useToast } from "@/hooks/use-toast";

// Config keys used by the application
const CONFIG_KEYS = {
  BACKEND: "backend",
  PRESET: "preset",
  MAX_ITERATIONS: "max_iterations",
  SCAN_DIRECTORIES: "scan_directories",
} as const;

export function SettingsDialog() {
  const [open, setOpen] = useState(false);
  const { toast } = useToast();

  // Fetch current config and options
  const { data: configData } = useConfig();
  const { data: backendsData } = useBackends();
  const { data: presetsData } = usePresets();
  const updateConfig = useUpdateConfig();

  // Local form state
  const [backend, setBackend] = useState("claude");
  const [preset, setPreset] = useState("default");
  const [maxIterations, setMaxIterations] = useState("100");
  const [scanDirectories, setScanDirectories] = useState("");

  // Update local state when config loads
  useEffect(() => {
    if (configData?.config) {
      setBackend(configData.config[CONFIG_KEYS.BACKEND] || "claude");
      setPreset(configData.config[CONFIG_KEYS.PRESET] || "default");
      setMaxIterations(configData.config[CONFIG_KEYS.MAX_ITERATIONS] || "100");
      setScanDirectories(configData.config[CONFIG_KEYS.SCAN_DIRECTORIES] || "");
    }
  }, [configData]);

  const handleSave = async () => {
    try {
      await updateConfig.mutateAsync({
        config: {
          [CONFIG_KEYS.BACKEND]: backend,
          [CONFIG_KEYS.PRESET]: preset,
          [CONFIG_KEYS.MAX_ITERATIONS]: maxIterations,
          [CONFIG_KEYS.SCAN_DIRECTORIES]: scanDirectories,
        },
      });
      toast({
        title: "Settings saved",
        description: "Your preferences have been updated.",
      });
      setOpen(false);
    } catch (error) {
      toast({
        title: "Failed to save settings",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    }
  };

  const backends = backendsData?.backends || [];
  const presets = presetsData?.presets || [];

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7 text-muted-foreground hover:text-foreground"
        >
          <Settings className="h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            Configure Ralph execution preferences.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          {/* AI Backend */}
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="backend" className="text-right">
              AI Backend
            </Label>
            <Select value={backend} onValueChange={setBackend}>
              <SelectTrigger id="backend" className="col-span-3">
                <SelectValue placeholder="Select backend" />
              </SelectTrigger>
              <SelectContent>
                {backends.map((b) => (
                  <SelectItem key={b.id} value={b.id}>
                    {b.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Preset */}
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="preset" className="text-right">
              Preset
            </Label>
            <Select value={preset} onValueChange={setPreset}>
              <SelectTrigger id="preset" className="col-span-3">
                <SelectValue placeholder="Select preset" />
              </SelectTrigger>
              <SelectContent>
                {presets.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Max Iterations */}
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="maxIterations" className="text-right">
              Max Iterations
            </Label>
            <Input
              id="maxIterations"
              type="number"
              value={maxIterations}
              onChange={(e) => setMaxIterations(e.target.value)}
              className="col-span-3"
              min="1"
              max="1000"
            />
          </div>

          {/* Scan Directories */}
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="scanDirs" className="text-right">
              Scan Dirs
            </Label>
            <Input
              id="scanDirs"
              value={scanDirectories}
              onChange={(e) => setScanDirectories(e.target.value)}
              placeholder="~/Projects, ~/Work"
              className="col-span-3"
            />
          </div>
          <p className="text-xs text-muted-foreground col-span-4 pl-[calc(25%+1rem)]">
            Comma-separated list of directories to scan for repositories.
          </p>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={updateConfig.isPending}>
            {updateConfig.isPending ? "Saving..." : "Save"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
