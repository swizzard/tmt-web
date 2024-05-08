import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { getUserTabs, renewToken, UserListTab, UserTabsResult } from "../api";
import useAuthToken from "../authToken";

export interface UserTabsProps {
  authToken?: string;
  setAuthToken: (token: string) => void;
}
export default function UserTabs() {
  const { authToken, setAuthToken } = useAuthToken();
  const [err, setErr] = useState<string | undefined>(undefined);
  const [userTabs, setUserTabs] = useState<UserListTab[]>([]);
  const [hasMore, setHasMore] = useState<boolean>(false);
  const [page, setPage] = useState<number>(1);

  const incrPage = () => setPage((page) => page + 1);
  const decrPage = () => setPage((page) => Math.max(0, page - 1));

  useEffect(() => {
    if (!authToken) return;
    getUserTabs({ authToken, page, pageSize: 1 })
      .then(({ results, has_more }: UserTabsResult) => {
        setUserTabs(results);
        setHasMore(has_more);
      })
      .catch((err) => {
        if (err?.isToken) {
          renewToken({ token: authToken })
            .then(({ access_token }) => {
              setAuthToken(access_token);
            })
            .catch((err) => {
              setErr(err.toString());
            });
        } else {
          setErr(err.toString());
        }
      });
  }, [page, authToken]);

  return (
    <div className="UserTabs">
      {err && <div className="error">{err}</div>}
      {!userTabs?.length ? (
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
            <tbody>{userTabs.map(UserListTabRow)}</tbody>
          </table>
          <div>
            {page > 1 && <button onClick={decrPage}>previous page</button>}
            {hasMore && <button onClick={incrPage}>next page</button>}
          </div>
        </div>
      )}
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
