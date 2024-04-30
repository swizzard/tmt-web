import React, { useEffect, useState } from "react";
import { SubmitHandler, useForm } from "react-hook-form";
import { authorize, LoginInput } from "./api";

export interface LoginProps {
  setAuthToken: (authToken: string | undefined) => void;
}
export default function Login({ setAuthToken }: LoginProps) {
  const {
    reset,
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<LoginInput>();
  const w = watch();
  const [err, setErr] = useState<string | undefined>();
  const onSubmit: SubmitHandler<LoginInput> = async (data) => {
    try {
      const { access_token } = await authorize(data);
      setAuthToken(access_token);
    } catch (e: any) {
      setErr(e.message ?? JSON.stringify(e));
      reset();
    }
  };
  useEffect(() => {
    console.log(errors);
  }, [errors]);

  useEffect(() => {
    setErr(undefined);
  }, [w.email, w.password]);
  return (
    <div className="Login">
      {err && <div className="err">{err}</div>}
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
