import { AuthorizedPost, PaginatedAuthorizedGet, TMTError } from ".";
import { API_URL, paginationDataToQuery } from "./_lib";

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

export async function createTab({
  authToken,
  data,
}: AuthorizedPost<NewTabInput>) {
  const endpoint = new URL("tabs", API_URL);
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
    body: JSON.stringify(data),
  });
  const responseData = await resp.json();
  if (!resp.ok) {
    throw new TMTError(responseData.error);
  } else {
    return responseData;
  }
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

export interface NewTab {
  user_id: string;
  url: string;
  notes?: string;
}

export interface MaybeNewTag {
  id?: string;
  user_id: string;
  tag: string;
}

export interface NewTabInput {
  tab: NewTab;
  tags: Array<MaybeNewTag>;
}

export type NewTabRequest = AuthorizedPost<NewTabInput>;
