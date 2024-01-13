import { atom } from "nanostores";
import { UserDTO } from "../../utils/apiTypes";

const apiHost: string = import.meta.env.VITE_API_HOST;

export const isLoadingUser = atom<boolean>(true);
export const currentUser = atom<UserDTO | undefined>(undefined);

isLoadingUser.set(true);

fetch(`${apiHost}/auth/me`, {
  credentials: "include",
})
  .then((res) => {
    if (res.status === 401) {
      window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}/dashboard`;
    }
  })
  .then((data) => {
    currentUser.set(data as unknown as UserDTO);
  })
  .catch((err) => {
    console.error(err);
  })
  .finally(() => {
    isLoadingUser.set(false);
  });
