import { Form, redirect } from "react-router-dom";
import { createTab, renewToken, type NewTabInput } from "../api";
import { getToken, setToken } from "../authToken";
import "./CreateTab.css";

export async function action({ request }: { request: Request }) {
  const authToken = getToken();
  if (!authToken) {
    throw new Error("Unauthorized");
  }
  const formData = await request.formData();
  const notes = formData.get("notes") as string | undefined;
  const url = formData.get("url") as string;
  const data: NewTabInput = {
    tab: {
      notes,
      url,
    },
    tags: [],
  };
  const f = async (tk: string) => {
    await createTab({ authToken: tk, data });
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

export default function CreateTab() {
  return (
    <div className="createTab">
      <Form method="post" action="/tabs/create">
        <label htmlFor="url">URL</label>
        <input type="url" id="url" name="url" />
        <label htmlFor="notes">Notes</label>
        <textarea rows={10} cols={50} id="notes" name="notes" />
        <div className="tags">
          <label htmlFor="tags">Tags</label>
          <select id="tags" name="tags" multiple></select>
        </div>
        <button type="submit">Create Tab</button>
      </Form>
    </div>
  );
}
