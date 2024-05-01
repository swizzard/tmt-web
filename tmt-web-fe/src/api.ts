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
export async function authorize({
  email: client_id,
  password: client_secret,
}: LoginInput): Promise<{ access_token: string }> {
  const endpoint = new URL("authorize", API_URL);
  const body = JSON.stringify({ client_id, client_secret });
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  return resp.json();
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
  notes: string;
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
  return resp.json();
}
