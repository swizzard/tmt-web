import { Form, Params, redirect, useLoaderData } from "react-router-dom";
import { getToken, setToken } from "../authToken";
import { deleteTab, renewToken, TabWithTags } from "../api";

export async function action({ params }: { params: Params }) {
  const authToken = getToken();
  if (!authToken) {
    throw new Error("Unauthorized");
  }
  const f = async (tk: string) => {
    await deleteTab(params.tabId!, { authToken: tk });
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

export default function DeleteTab() {
  const tab = useLoaderData() as TabWithTags;
  return (
    <div className="deleteTab">
      <h1>Delete Tab?</h1>
      <Form method="post">
        <label htmlFor="url">URL</label>
        <input
          type="url"
          id="url"
          name="url"
          readOnly={true}
          value={tab.tab.url}
        />
        <label htmlFor="notes">Notes</label>
        <textarea
          id="notes"
          name="notes"
          readOnly={true}
          value={tab.tab.notes}
        />
        <label htmlFor="tags">Tags</label>
        <select id="tags" name="tags" multiple={true} disabled={true}></select>
        <button type="submit">Delete Tab</button>
      </Form>
    </div>
  );
}
