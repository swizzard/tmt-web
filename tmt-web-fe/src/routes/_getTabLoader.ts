import { getToken, setToken } from "../authToken";
import { getTabDetails, renewToken, TabWithTags } from "../api";
import { Params } from "react-router-dom";

export default async function getTabDetailsLoader({
  params,
}: {
  params: Params;
}): Promise<TabWithTags> {
  const authToken = getToken();
  if (!authToken) {
    throw new Error("Unauthorized");
  }
  const f = async (tk: string) =>
    getTabDetails(params.tabId!, { authToken: tk });
  try {
    return f(authToken);
  } catch (e: any) {
    if (e.isToken) {
      const { access_token } = await renewToken({ token: authToken });
      setToken(access_token);
      return f(getToken()!);
    } else {
      throw e;
    }
  }
}
