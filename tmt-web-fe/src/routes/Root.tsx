import { Link, Outlet } from "react-router-dom";
import "./Root.css";

function Root({ loggedIn }: { loggedIn: boolean }) {
  return (
    <>
      <div id="top-bar">
        <h1>Too Many Tabs</h1>
      </div>
      <div id="content-wrapper">
        {loggedIn && <LoggedInSidebar />}
        <div id="content">
          <Outlet />
        </div>
      </div>
      <div id="footer"></div>
    </>
  );
}

function LoggedInSidebar() {
  return (
    <div id="side-bar">
      <div className="nav vertical">
        <Link to="/tabs/personal/1">My Tabs</Link>
        <Link to="/tabs/create">New Tab</Link>
        <Link to="/logout">Logout</Link>
      </div>
    </div>
  );
}
export function LoggedInRoot() {
  return <Root loggedIn={true} />;
}
export function LoggedOutRoot() {
  return <Root loggedIn={false} />;
}
