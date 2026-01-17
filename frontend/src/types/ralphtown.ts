import type { Session, SessionDetails, Message, Repo } from "@/api/types";

export type AgentStatus = "running" | "completed" | "error" | "pending" | "idle" | "cancelled";

export interface ChatMessage {
  id: string;
  role: "user" | "agent";
  content: string;
  timestamp: Date;
}

export interface RalphtownInstance {
  id: string;
  title: string;
  repo: string;
  repoId: string;
  branch: string;
  status: AgentStatus;
  createdAt: Date;
  linesAdded?: number;
  linesRemoved?: number;
  model: string;
  messages: ChatMessage[];
}

export interface Repository {
  id: string;
  fullName: string;
  owner: string;
  name: string;
  path: string;
  defaultBranch: string;
  branches: string[];
}

// --- Adapters to convert API types to UI types ---

export function mapApiMessageToChat(msg: Message): ChatMessage {
  return {
    id: msg.id,
    role: msg.role === "user" ? "user" : "agent",
    content: msg.content,
    timestamp: new Date(msg.created_at),
  };
}

export function mapApiSessionToInstance(
  session: Session | SessionDetails,
  repo: Repo | undefined,
  currentBranch?: string
): RalphtownInstance {
  const messages = "messages" in session ? session.messages : [];
  return {
    id: session.id,
    title: session.name || "Untitled Session",
    repo: repo?.name || "Unknown",
    repoId: session.repo_id,
    branch: currentBranch || "main",
    status: session.status as AgentStatus,
    createdAt: new Date(session.created_at),
    model: "Claude Code", // TODO: Store model in session
    messages: messages.map(mapApiMessageToChat),
  };
}

export function mapApiRepoToRepository(repo: Repo, branches?: string[]): Repository {
  const name = repo.name;
  return {
    id: repo.id,
    fullName: name, // In local repos, we just use the name
    owner: "", // No owner for local repos
    name: name,
    path: repo.path,
    defaultBranch: "main", // Will be updated when we fetch branches
    branches: branches || ["main"],
  };
}
