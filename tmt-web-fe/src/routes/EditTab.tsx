import { Form, Params, redirect, useLoaderData } from "react-router-dom";
import { renewToken, updateTab, TabWithTags, UpdateTabInput } from "../api";
import { getToken, setToken } from "../authToken";
import "./EditTab.css";
import TabForm from "./_tabForm";

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
        <TabForm tab={tab} />
        <button type="submit">Update Tab</button>
      </Form>
    </div>
  );
}
