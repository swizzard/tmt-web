import {
  AuthorizedDelete,
  AuthorizedGet,
  AuthorizedPost,
  PaginatedAuthorizedGet,
  TMTError,
} from ".";
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
  const endpoint = new URL("tabs/with-tags", API_URL);
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
    body: JSON.stringify(data),
  });
  try {
    const responseData = await resp.json();
    if (!resp.ok) {
      throw new TMTError(responseData.error);
    } else {
      return responseData;
    }
  } catch (_e: any) {
    console.log(resp.body);
    throw new TMTError("Unknown error");
  }
}

export async function deleteTab(
  tabId: string,
  { authToken }: AuthorizedDelete,
) {
  const endpoint = new URL(`tabs/${tabId}`, API_URL);
  const resp = await fetch(endpoint.href, {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
  });
  if (!resp.ok) {
    throw new TMTError(resp.statusText);
  } else {
    return;
  }
}

export async function updateTab(
  tabId: string,
  { authToken, data }: AuthorizedPost<UpdateTabInput>,
) {
  const endpoint = new URL(`tabs/${tabId}`, API_URL);
  const resp = await fetch(endpoint.href, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
    body: JSON.stringify(data),
  });
  try {
    const responseData = await resp.json();
    if (!resp.ok) {
      throw new TMTError(responseData.error);
    } else {
      return responseData;
    }
  } catch (_e: any) {
    console.log(resp.body);
    throw new TMTError("Unknown error");
  }
}

export async function getTabDetails(
  tabId: string,
  { authToken }: AuthorizedGet,
): Promise<TabWithTags> {
  const endpoint = new URL(`tabs/${tabId}/with-tags`, API_URL);
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

export interface UpdateTab {
  url: string;
  notes?: string;
}

export interface UpdateTabInput {
  tab: UpdateTab;
  tags: Array<MaybeNewTag>;
}

export type UpdateTabRequest = AuthorizedPost<UpdateTabInput>;

export type Tab = {
  id: string;
  user_id: string;
  url: string;
  notes?: string;
};
export type Tag = {
  id: string;
  user_id: string;
  tag: string;
};
export type TabWithTags = {
  tab: Tab;
  tags: Array<Tag>;
};
