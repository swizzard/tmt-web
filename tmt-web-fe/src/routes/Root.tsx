import useAuthToken from "../authToken";
import { Link, Outlet } from "react-router-dom";
import "./Root.css";

export default function Root() {
  const { authToken } = useAuthToken();
  return (
    <>
      <div id="top-bar">
        <h1>Too Many Tabs</h1>
      </div>
      <div id="content-wrapper">
        {authToken && <LoggedInSidebar />}
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
      <ul className="nav vertical">
        <li>
          <Link to="/">Home</Link>
        </li>
        <li>
          <Link to="/tabs/personal/1">Your Tabs</Link>
        </li>
        <li>
          <Link to="/logout">Logout</Link>
        </li>
      </ul>
    </div>
  );
}
