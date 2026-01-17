import { RalphtownInstance, Repository, ChatMessage } from "@/types/ralphtown";

export const mockRepositories: Repository[] = [
  {
    id: "1",
    fullName: "pRizz/degen-server",
    owner: "pRizz",
    name: "degen-server",
    defaultBranch: "main",
    branches: ["main", "develop", "feature/api-v2"],
  },
  {
    id: "2",
    fullName: "pRizz/text-2-audiobook",
    owner: "pRizz",
    name: "text-2-audiobook",
    defaultBranch: "main",
    branches: ["main", "develop"],
  },
  {
    id: "3",
    fullName: "pRizz/ralphtown",
    owner: "pRizz",
    name: "ralphtown",
    defaultBranch: "main",
    branches: ["main", "feature/agents"],
  },
  {
    id: "4",
    fullName: "pRizz/install-basic-git-hooks",
    owner: "pRizz",
    name: "install-basic-git-hooks",
    defaultBranch: "main",
    branches: ["main"],
  },
];

const createMockMessages = (title: string, createdAt: Date): ChatMessage[] => [
  {
    id: "1",
    role: "user",
    content: title,
    timestamp: createdAt,
  },
  {
    id: "2",
    role: "agent",
    content: `I'll help you with "${title}". Let me analyze the codebase and identify the relevant files...

I found several files that need to be modified. I'm now implementing the changes.`,
    timestamp: new Date(createdAt.getTime() + 30000),
  },
  {
    id: "3",
    role: "agent",
    content: `I've made the following changes:

• Updated \`src/api/handlers.ts\` - Added new endpoint handlers
• Modified \`src/utils/validation.ts\` - Improved input validation
• Created \`src/tests/api.test.ts\` - Added unit tests

The changes have been committed and are ready for review.`,
    timestamp: new Date(createdAt.getTime() + 120000),
  },
];

export const mockRalphtownInstances: RalphtownInstance[] = [
  {
    id: "1",
    title: "Public API readiness",
    repo: "degen-server",
    branch: "main",
    status: "completed",
    createdAt: new Date(Date.now() - 20 * 60 * 60 * 1000),
    model: "GPT-5.2 Codex High",
    messages: createMockMessages("Public API readiness", new Date(Date.now() - 20 * 60 * 60 * 1000)),
  },
  {
    id: "2",
    title: "Readme beads installation",
    repo: "ralphtown",
    branch: "main",
    status: "running",
    createdAt: new Date(Date.now() - 24 * 60 * 60 * 1000),
    linesAdded: 3,
    model: "GPT-5.2 Codex High",
    messages: [
      {
        id: "1",
        role: "user",
        content: "Readme beads installation",
        timestamp: new Date(Date.now() - 24 * 60 * 60 * 1000),
      },
      {
        id: "2",
        role: "agent",
        content: "I'm currently updating the README with installation instructions for beads. Analyzing the project structure...",
        timestamp: new Date(Date.now() - 24 * 60 * 60 * 1000 + 30000),
      },
    ],
  },
  {
    id: "3",
    title: "Tts engine capability feedback",
    repo: "text-2-audiobook",
    branch: "main",
    status: "completed",
    createdAt: new Date(Date.now() - 30 * 60 * 60 * 1000),
    linesAdded: 97,
    linesRemoved: 33,
    model: "GPT-5.2 Codex High",
    messages: createMockMessages("Tts engine capability feedback", new Date(Date.now() - 30 * 60 * 60 * 1000)),
  },
  {
    id: "4",
    title: "Speech generation progress di...",
    repo: "text-2-audiobook",
    branch: "main",
    status: "completed",
    createdAt: new Date(Date.now() - 36 * 60 * 60 * 1000),
    linesAdded: 295,
    linesRemoved: 12,
    model: "GPT-5.2 Codex High",
    messages: createMockMessages("Speech generation progress display", new Date(Date.now() - 36 * 60 * 60 * 1000)),
  },
  {
    id: "5",
    title: "M4b export file size",
    repo: "text-2-audiobook",
    branch: "main",
    status: "pending",
    createdAt: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000),
    linesAdded: 101,
    linesRemoved: 67,
    model: "GPT-5.2 Codex High",
    messages: createMockMessages("M4b export file size optimization", new Date(Date.now() - 2 * 24 * 60 * 60 * 1000)),
  },
  {
    id: "6",
    title: "Recursive uninstall hooks com...",
    repo: "install-basic-git-hooks",
    branch: "main",
    status: "error",
    createdAt: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000),
    linesAdded: 596,
    linesRemoved: 116,
    model: "GPT-5.2 Codex High",
    messages: [
      ...createMockMessages("Recursive uninstall hooks command", new Date(Date.now() - 3 * 24 * 60 * 60 * 1000)),
      {
        id: "4",
        role: "agent",
        content: "❌ Error: Failed to complete the task. The recursive operation encountered a circular dependency that couldn't be resolved automatically.",
        timestamp: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000 + 180000),
      },
    ],
  },
];

export const availableModels = [
  "GPT-5.2 Codex High",
  "GPT-5.2 Codex",
  "Claude 4 Opus",
  "Claude 4 Sonnet",
  "Gemini 2.5 Pro",
];

export const quickActions = [
  "Run security audit",
  "Improve AGENTS.md",
  "Solve a TODO",
];
