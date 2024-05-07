const API_URL = new URL("http://0.0.0.0:8080/");

export interface AuthorizedPost<T> {
  authToken: string;
  data: T;
}

export interface AuthorizedGet {
  authToken: string;
}

export interface PaginatedAuthorizedGet {
  authToken: string;
  page?: number;
  pageSize?: number;
}

export interface LoginInput {
  email: string;
  password: string;
}

export interface AuthResponse {
  access_token: string;
}
export async function authorize({
  email: client_id,
  password: client_secret,
}: LoginInput): Promise<AuthResponse> {
  const endpoint = new URL("authorize", API_URL);
  const body = JSON.stringify({ client_id, client_secret });
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  const data = await resp.json();
  if (resp.ok) {
    return data;
  } else {
    throw new TMTError(data.error);
  }
}

export async function logout({
  authToken,
}: AuthorizedPost<{}>): Promise<{ ok: boolean }> {
  const endpoint = new URL("logout", API_URL);
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
  });
  return resp.json();
}

export interface UserListTab {
  id: string;
  user_id: string;
  url: string;
}

export interface UserTabsResult {
  has_more: boolean;
  results: UserListTab[];
}

function paginationDataToQuery(data?: {
  page?: number;
  pageSize?: number;
}): string | undefined {
  if (!data) return undefined;
  let pp: Record<string, string> = {};
  let anyData = false;
  if (data.page) {
    pp.page = data.page.toString();
    anyData = true;
  }
  if (data.pageSize) {
    pp.page_size = data.pageSize.toString();
    anyData = true;
  }
  if (!anyData) {
    return undefined;
  }
  return new URLSearchParams(pp).toString();
}
export async function getUserTabs({
  authToken,
  ...paginationData
}: PaginatedAuthorizedGet): Promise<UserTabsResult> {
  const endpoint = new URL("users/tabs", API_URL);
  const params = paginationDataToQuery(paginationData);
  if (params) {
    endpoint.search = params;
  }
  const resp = await fetch(endpoint.href, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
  });
  const data = await resp.json();
  if (!resp.ok) {
    throw new TMTError(data.error);
  } else {
    return data;
  }
}

interface RenewRequest {
  token: string;
}

export async function renewToken({
  token,
}: RenewRequest): Promise<AuthResponse> {
  const endpoint = new URL("renew", API_URL);
  const body = JSON.stringify({ token });
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  const data = await resp.json();
  if (!resp.ok) {
    throw new TMTError(data.error);
  } else {
    return data;
  }
}

export class TMTError extends Error {
  isToken: boolean;
  constructor(message: string) {
    super(message);
    this.isToken = TMTError.isTokenError(message);
  }
  static isTokenError(message: string): boolean {
    return message === "Invalid token";
  }
}
