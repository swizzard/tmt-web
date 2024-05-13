import { Link } from "react-router-dom";
import useAuthToken from "../authToken";
import "./Home.css";
export default function Home() {
  const { authToken } = useAuthToken();
  return (
    <div className="Home">
      {authToken ? (
        <div>
          <h1>Too Many Tabs</h1>
          <p>You are logged in.</p>
          <p>
            <Link to="/tabs/personal/1">
              <button type="button" id="yourTabs">
                Your Tabs
              </button>
            </Link>
          </p>
          <p>
            <Link to="/logout">
              <button type="button" id="logout">
                Logout
              </button>
            </Link>
          </p>
        </div>
      ) : (
        <div>
          <h1>Welcome!</h1>
          <p>You are not logged in.</p>
          <p>
            <Link to="/login">
              <button type="button">Login</button>
            </Link>
          </p>
        </div>
      )}
    </div>
  );
}
