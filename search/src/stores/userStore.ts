import { atom } from "nanostores";
import { isUserDTO, type UserDTO } from "../../utils/apiTypes";

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
const apiHost: string = import.meta.env.VITE_API_HOST;

export const isLoadingUser = atom<boolean>(true);
export const currentUser = atom<UserDTO | undefined>(undefined);

fetch(`${apiHost}/auth/me`, {
  method: "GET",
  headers: {
    "Content-Type": "application/json",
  },
  credentials: "include",
})
  .then((response) => {
    if (response.ok) {
      void response.json().then((data) => {
        if (isUserDTO(data)) {
          currentUser.set(data);
        }
        isLoadingUser.set(false);
      });
      return;
    }
    isLoadingUser.set(false);
  })
  .catch(() => {
    isLoadingUser.set(false);
  });
