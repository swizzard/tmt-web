import { Link, Params, useLoaderData } from "react-router-dom";
import { getUserTabs, renewToken, UserListTab, UserTabsResult } from "../api";
import { getToken, setToken } from "../authToken";

export type UserTabsLoaderResult = UserTabsResult & { page: number };
export async function loader({
  params,
}: {
  params: Params;
}): Promise<UserTabsLoaderResult> {
  const authToken = getToken();
  if (!authToken) {
    throw new Error("Unauthorized");
  }
  const page = parseInt(params.page || "1", 10);
  console.log("loader page", page);
  const f = async (tk: string) =>
    getUserTabs({ authToken: tk, page, pageSize: 1 });
  let resp: UserTabsResult | undefined = undefined;
  try {
    resp = await f(authToken);
  } catch (e: any) {
    if (e.isToken) {
      const { access_token } = await renewToken({ token: authToken });
      setToken(access_token);
      resp = await f(getToken()!);
    } else {
      throw e;
    }
  }
  return { page, ...resp };
}

export default function UserTabs() {
  const {
    page,
    has_more: hasMore,
    results,
  } = useLoaderData() as UserTabsLoaderResult;
  console.log("component page", page);

  return (
    <div className="UserTabs">
      {!results?.length ? (
        <div>No Tabs</div>
      ) : (
        <div>
          <table>
            <thead>
              <tr>
                <th>actions</th>
                <th>url</th>
              </tr>
            </thead>
            <tbody>{results.map(UserListTabRow)}</tbody>
          </table>
          <div>
            {page > 1 && (
              <Link to={`../personal/${page - 1}`} reloadDocument={true}>
                prev page
              </Link>
            )}
            {hasMore ? (
              page === 1 ? (
                <Link to={`../personal/2`} reloadDocument={true}>
                  next page
                </Link>
              ) : (
                <Link to={`../personal/${page + 1}`} reloadDocument={true}>
                  next page
                </Link>
              )
            ) : null}
          </div>
        </div>
      )}
      <div>
        <Link to="/tabs/create">Create Tab</Link>
      </div>
      <div>
        <Link to="/logout">Logout</Link>
      </div>
    </div>
  );
}

function UserListTabRow({ id, url }: UserListTab) {
  return (
    <tr key={id}>
      <td>
        <Link to={`/tabs/${id}/edit`}>edit</Link>
        <Link to={`/tabs/${id}/delete`}>delete</Link>
      </td>
      <td>
        <a href={url}>{url}</a>
      </td>
    </tr>
  );
}
