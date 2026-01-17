// API client with fetch wrappers and error handling

import type {
  Repo,
  AddRepoRequest,
  ScanRequest,
  ScanResponse,
  Session,
  SessionDetails,
  CreateSessionRequest,
  RunSessionRequest,
  RunSessionResponse,
  CancelSessionResponse,
  OutputResponse,
  GitStatusResponse,
  GitLogResponse,
  GitBranchesResponse,
  GitDiffResponse,
  GitCommandResponse,
  CommitRequest,
  ResetRequest,
  CheckoutRequest,
} from "./types";

const API_BASE = "/api";

export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    public body?: string
  ) {
    super(`API Error ${status}: ${statusText}`);
    this.name = "ApiError";
  }
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
  });

  if (!response.ok) {
    const body = await response.text().catch(() => undefined);
    throw new ApiError(response.status, response.statusText, body);
  }

  // Handle empty responses (204 No Content or empty body)
  const text = await response.text();
  if (!text) {
    return undefined as T;
  }

  return JSON.parse(text) as T;
}

// --- Repos ---

export async function listRepos(): Promise<Repo[]> {
  return request<Repo[]>("/repos");
}

export async function addRepo(req: AddRepoRequest): Promise<Repo> {
  return request<Repo>("/repos", {
    method: "POST",
    body: JSON.stringify(req),
  });
}

export async function deleteRepo(id: string): Promise<void> {
  await request<void>(`/repos/${id}`, { method: "DELETE" });
}

export async function scanRepos(req: ScanRequest): Promise<ScanResponse> {
  return request<ScanResponse>("/repos/scan", {
    method: "POST",
    body: JSON.stringify(req),
  });
}

// --- Sessions ---

export async function listSessions(): Promise<Session[]> {
  return request<Session[]>("/sessions");
}

export async function getSession(id: string): Promise<SessionDetails> {
  return request<SessionDetails>(`/sessions/${id}`);
}

export async function createSession(req: CreateSessionRequest): Promise<Session> {
  return request<Session>("/sessions", {
    method: "POST",
    body: JSON.stringify(req),
  });
}

export async function deleteSession(id: string): Promise<void> {
  await request<void>(`/sessions/${id}`, { method: "DELETE" });
}

export async function runSession(
  id: string,
  req: RunSessionRequest
): Promise<RunSessionResponse> {
  return request<RunSessionResponse>(`/sessions/${id}/run`, {
    method: "POST",
    body: JSON.stringify(req),
  });
}

export async function cancelSession(id: string): Promise<CancelSessionResponse> {
  return request<CancelSessionResponse>(`/sessions/${id}/cancel`, {
    method: "POST",
  });
}

export async function getSessionOutput(
  id: string,
  params?: { stream?: "stdout" | "stderr"; limit?: number; offset?: number }
): Promise<OutputResponse> {
  const searchParams = new URLSearchParams();
  if (params?.stream) searchParams.set("stream", params.stream);
  if (params?.limit) searchParams.set("limit", String(params.limit));
  if (params?.offset) searchParams.set("offset", String(params.offset));

  const query = searchParams.toString();
  return request<OutputResponse>(`/sessions/${id}/output${query ? `?${query}` : ""}`);
}

// --- Git ---

export async function getGitStatus(sessionId: string): Promise<GitStatusResponse> {
  return request<GitStatusResponse>(`/sessions/${sessionId}/git/status`);
}

export async function getGitLog(
  sessionId: string,
  limit?: number
): Promise<GitLogResponse> {
  const query = limit ? `?limit=${limit}` : "";
  return request<GitLogResponse>(`/sessions/${sessionId}/git/log${query}`);
}

export async function getGitBranches(sessionId: string): Promise<GitBranchesResponse> {
  return request<GitBranchesResponse>(`/sessions/${sessionId}/git/branches`);
}

export async function getGitDiff(sessionId: string): Promise<GitDiffResponse> {
  return request<GitDiffResponse>(`/sessions/${sessionId}/git/diff`);
}

export async function gitPull(sessionId: string): Promise<GitCommandResponse> {
  return request<GitCommandResponse>(`/sessions/${sessionId}/git/pull`, {
    method: "POST",
  });
}

export async function gitPush(sessionId: string): Promise<GitCommandResponse> {
  return request<GitCommandResponse>(`/sessions/${sessionId}/git/push`, {
    method: "POST",
  });
}

export async function gitCommit(
  sessionId: string,
  req: CommitRequest
): Promise<GitCommandResponse> {
  return request<GitCommandResponse>(`/sessions/${sessionId}/git/commit`, {
    method: "POST",
    body: JSON.stringify(req),
  });
}

export async function gitReset(
  sessionId: string,
  req: ResetRequest
): Promise<GitCommandResponse> {
  return request<GitCommandResponse>(`/sessions/${sessionId}/git/reset`, {
    method: "POST",
    body: JSON.stringify(req),
  });
}

export async function gitCheckout(
  sessionId: string,
  req: CheckoutRequest
): Promise<GitCommandResponse> {
  return request<GitCommandResponse>(`/sessions/${sessionId}/git/checkout`, {
    method: "POST",
    body: JSON.stringify(req),
  });
}
