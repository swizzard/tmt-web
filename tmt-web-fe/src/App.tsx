import React, { useEffect, useState } from "react";
import { useForm, SubmitHandler } from "react-hook-form";
import "./App.css";

const API_URL = new URL("http://0.0.0.0:8080/");

interface LoginInput {
  email: string;
  password: string;
}

function App() {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginInput>();
  const [authToken, setAuthToken] = useState<string | undefined>();
  const onSubmit: SubmitHandler<LoginInput> = async (data) => {
    const endpoint = new URL("authorize", API_URL);
    const body = JSON.stringify(data);
    console.log(`posting to ${endpoint} ${body}`);
    const resp = await fetch(endpoint.href, {
      method: "POST",
      mode: "no-cors",
      headers: {
        "Content-Type": "application/json",
      },
      body,
    });
    debugger;
    const json = await resp.json();
    setAuthToken(json.token);
  };

  useEffect(() => {
    console.log(errors);
  }, [errors]);

  useEffect(() => {
    console.log(`got authToken: ${authToken}`);
  }, [authToken]);

  return (
    <div className="App">
      <form onSubmit={handleSubmit(onSubmit)}>
        <label htmlFor="email">Email</label>
        <input type="email" id="email" {...register("email")} />
        <label htmlFor="password">Password</label>
        <input type="password" id="password" {...register("password")} />
        <input type="submit" />
      </form>
    </div>
  );
}

export default App;
