import { Link } from "react-router-dom";
import useAuthToken from "../authToken";
export default function Root() {
  const { authToken } = useAuthToken();
  return (
    <div className="App">
      {authToken ? (
        <div>
          <h1>Welcome!</h1>
          <p>You are logged in.</p>
          <p>
            <Link to="/tabs/personal/1">Your Tabs</Link>
          </p>
          <p>
            <Link to="/logout">Logout</Link>
          </p>
        </div>
      ) : (
        <div>
          <h1>Welcome!</h1>
          <p>You are not logged in.</p>
          <p>
            <Link to="/login">Login</Link>
          </p>
        </div>
      )}
    </div>
  );
}
