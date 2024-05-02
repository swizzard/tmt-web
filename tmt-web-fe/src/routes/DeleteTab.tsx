export interface DeleteTabProps {
  authToken?: string;
  setAuthToken: (token: string) => void;
}

export default function DeleteTab(_props: DeleteTabProps) {
  console.log("delete tab");
  return <div>DeleteTab</div>;
}
