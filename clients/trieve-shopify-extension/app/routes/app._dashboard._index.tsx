import { useTrieve } from "app/context/trieveContext";

export default function Dashboard() {
  const { dataset, trieve } = useTrieve();
  return <div>Homepage Dataset: {dataset.name}</div>;
}
