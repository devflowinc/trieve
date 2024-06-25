import { useContext } from "solid-js";
import { UserContext } from "../contexts/UserAuthContext";

export const Home = () => {
  const user = useContext(UserContext);
  return (
    <div>
      <div>Home apge</div>
      {JSON.stringify(user?.user)}
    </div>
  );
};
