import { useContext } from "solid-js";
import { OrgContext } from "../contexts/OrgDatasetContext";

export const Home = () => {
  const user = useContext(OrgContext);
  return (
    <div>
      <div>Home apge</div>
    </div>
  );
};
