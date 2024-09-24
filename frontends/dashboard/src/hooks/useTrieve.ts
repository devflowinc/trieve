import { useContext } from "solid-js";
import { ApiContext } from "../api/trieve";

export const useTrieve = () => {
  const trieve = useContext(ApiContext);
  return trieve;
};
