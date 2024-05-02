import React, { useEffect, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { getUserTabs, renewToken, UserListTab, UserTabsResult } from "../api";

export interface UserTabsProps {
  authToken?: string;
  setAuthToken: (token: string) => void;
}
export default function UserTabs({ setAuthToken, authToken }: UserTabsProps) {
  const { state } = useLocation();
  const [err, setErr] = useState<string | undefined>(undefined);
  const [userTabs, setUserTabs] = useState<UserListTab[]>([]);
  const [hasMore, setHasMore] = useState<boolean>(false);
  const [page, setPage] = useState<number>(1);

  const incrPage = () => setPage((page) => page + 1);
  const decrPage = () => setPage((page) => Math.max(0, page - 1));

  useEffect(() => {
    const token = authToken ?? state?.authToken;
    if (!token) return;
    getUserTabs({ authToken: token, page, pageSize: 1 })
      .then(({ results, has_more }: UserTabsResult) => {
        setUserTabs(results);
        setHasMore(has_more);
      })
      .catch((err) => {
        if (err?.isToken) {
          renewToken({ token })
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
  }, [page, authToken, state.authToken]);

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
            <tbody>
              {userTabs.map((tab) => (
                <tr key={tab.id}>
                  <td>
                    <button>edit</button>
                    <button>delete</button>
                  </td>
                  <td>{tab.url}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <div>
            {page > 0 && <button onClick={decrPage}>previous page</button>}
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
