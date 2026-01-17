import { useState } from "react";
import { Image, ArrowUp, ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { availableModels, quickActions } from "@/data/mockData";

interface PromptInputProps {
  onSubmit: (prompt: string, model: string) => void;
}

export function PromptInput({ onSubmit }: PromptInputProps) {
  const [prompt, setPrompt] = useState("");
  const [selectedModel, setSelectedModel] = useState(availableModels[0]);

  const handleSubmit = () => {
    if (prompt.trim()) {
      onSubmit(prompt, selectedModel);
      setPrompt("");
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      handleSubmit();
    }
  };

  return (
    <div className="w-full max-w-2xl">
      {/* Prompt Box */}
      <div className="bg-card border border-border rounded-xl overflow-hidden">
        <Textarea
          placeholder="Ask Ralph to build, fix bugs, explore"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={handleKeyDown}
          className="min-h-[100px] border-0 bg-transparent resize-none focus-visible:ring-0 text-base px-4 py-3"
        />

        {/* Bottom bar */}
        <div className="flex items-center justify-between px-3 py-2 border-t border-border">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                className="h-8 px-2 text-sm text-muted-foreground hover:text-foreground gap-1.5"
              >
                {selectedModel}
                <ChevronDown className="h-3.5 w-3.5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              {availableModels.map((model) => (
                <DropdownMenuItem
                  key={model}
                  onClick={() => setSelectedModel(model)}
                  className="cursor-pointer"
                >
                  {model}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-muted-foreground hover:text-foreground"
            >
              <Image className="h-4 w-4" />
            </Button>
            <Button
              size="icon"
              className="h-8 w-8 rounded-full bg-muted hover:bg-accent"
              onClick={handleSubmit}
              disabled={!prompt.trim()}
            >
              <ArrowUp className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="flex items-center justify-center gap-2 mt-4">
        {quickActions.map((action) => (
          <Button
            key={action}
            variant="outline"
            size="sm"
            className="rounded-full text-xs border-border text-muted-foreground hover:text-foreground"
            onClick={() => setPrompt(action)}
          >
            {action}
          </Button>
        ))}
      </div>
    </div>
  );
}
