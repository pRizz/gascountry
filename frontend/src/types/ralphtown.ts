export type AgentStatus = "running" | "completed" | "error" | "pending";

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
  defaultBranch: string;
  branches: string[];
}
