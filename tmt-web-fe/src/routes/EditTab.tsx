import { Form, Params, redirect, useLoaderData } from "react-router-dom";
import { renewToken, updateTab, TabWithTags, UpdateTabInput } from "../api";
import { getToken, setToken } from "../authToken";

export async function action({
  request,
  params,
}: {
  request: Request;
  params: Params;
}) {
  const authToken = getToken();
  if (!authToken) {
    throw new Error("Unauthorized");
  }
  const formData = await request.formData();
  const notes = formData.get("notes") as string | undefined;
  const url = formData.get("url") as string;
  const data: UpdateTabInput = {
    tab: {
      notes,
      url,
    },
    tags: [],
  };
  const f = async (tk: string) => {
    await updateTab(params.tabId!, { authToken: tk, data });
    return redirect("/tabs");
  };
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

export default function EditTab() {
  const tab = useLoaderData() as TabWithTags;
  return (
    <div className="editTab">
      <Form method="post">
        <label htmlFor="url">URL</label>
        <input type="url" id="url" name="url" defaultValue={tab.tab.url} />
        <label htmlFor="notes">Notes</label>
        <textarea id="notes" name="notes" defaultValue={tab.tab.notes} />
        <label htmlFor="tags">Tags</label>
        <select id="tags" name="tags" multiple></select>
        <button type="submit">Update Tab</button>
      </Form>
    </div>
  );
}
