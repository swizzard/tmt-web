import { Link, Params, useLoaderData } from "react-router-dom";
import { getUserTabs, renewToken, UserListTab, UserTabsResult } from "../api";
import { getToken, setToken } from "../authToken";
import { FaArrowRight, FaArrowLeft } from "react-icons/fa";
import "./UserTabs.css";

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
    getUserTabs({ authToken: tk, page, pageSize: 10 });
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
                <th colSpan={2}>actions</th>
                <th>url</th>
              </tr>
            </thead>
            <tbody>{results.map(UserListTabRow)}</tbody>
          </table>
          <div>
            {page > 1 && (
              <Link to={`../personal/${page - 1}`} reloadDocument={true}>
                <button className="navButton" type="button">
                  <FaArrowLeft /> prev page
                </button>
              </Link>
            )}
            {hasMore ? (
              page === 1 ? (
                <Link to={`../personal/2`} reloadDocument={true}>
                  <button className="navButton" type="button">
                    <FaArrowRight /> next page
                  </button>
                </Link>
              ) : (
                <Link to={`../personal/${page + 1}`} reloadDocument={true}>
                  <button className="navButton" type="button">
                    <FaArrowRight /> next page
                  </button>
                </Link>
              )
            ) : null}
          </div>
        </div>
      )}
      <div className="buttonWrapper create">
        <Link to="/tabs/create">
          <button id="createTabButton" type="button">
            Create Tab
          </button>
        </Link>
      </div>
      <div className="buttonWrapper logout">
        <Link to="/logout">
          <button id="logoutButton" type="button">
            Logout
          </button>
        </Link>
      </div>
    </div>
  );
}

function UserListTabRow({ id, url }: UserListTab) {
  return (
    <tr key={id}>
      <td>
        <Link to={`/tabs/edit/${id}`}>
          <button className="editTab" type="button">
            edit
          </button>
        </Link>
      </td>
      <td>
        <Link to={`/tabs/delete/${id}`}>
          <button className="deleteTab" type="button">
            delete
          </button>
        </Link>
      </td>
      <td>
        <a href={url}>{url}</a>
      </td>
    </tr>
  );
}
