import {
  Popover,
  PopoverButton,
  PopoverPanel,
  Menu,
  MenuItem,
} from "solid-headless";
import {
  isFileUploadCompleteNotificationDTO,
  type NotificationDTO,
  type UserDTO,
  isVerificationNotificationDTO,
  NotificationWithPagesDTO,
} from "../../../utils/apiTypes";
import { IoNotificationsOutline } from "solid-icons/io";
import { Show, createEffect, createSignal, For } from "solid-js";
import { VsCheckAll } from "solid-icons/vs";
import { BiRegularChevronLeft, BiRegularChevronRight } from "solid-icons/bi";
import { SingleNotification } from "../Notification";

export const NotificationPopover = (props: { user: UserDTO | null }) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const similarityScoreThreshold =
    (import.meta.env.SIMILARITY_SCORE_THRESHOLD as number | undefined) ?? 80;

  const [notifs, setNotifs] = createSignal<NotificationDTO[]>([]);
  const [page, setPage] = createSignal(1);
  const [totalPages, setTotalPages] = createSignal(0);
  const [usingPanel, setUsingPanel] = createSignal(false);

  createEffect(() => {
    fetchNotifs();
  });

  const fetchNotifs = () => {
    void fetch(`${apiHost}/notifications/${page()}`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      void response.json().then((data) => {
        if (response.ok) {
          const notifData = data as NotificationWithPagesDTO;
          setNotifs(notifData.notifications);
          setTotalPages(notifData.total_pages);
        }
      });
    });
  };

  const markAsRead = (notification: NotificationDTO) => {
    void fetch(`${apiHost}/notifications`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({
        notification_id: notification.id,
      }),
    }).then((response) => {
      if (response.ok) {
        const isVerif = isVerificationNotificationDTO(notification);
        const isFileUpload = isFileUploadCompleteNotificationDTO(notification);
        setNotifs((prev_notifs) =>
          prev_notifs.map((notif) => {
            if (isVerif && isVerificationNotificationDTO(notif)) {
              if (notification.card_uuid === notif.card_uuid) {
                notif.user_read = true;
              }
            } else if (
              isFileUpload &&
              isFileUploadCompleteNotificationDTO(notif)
            ) {
              if (notification.collection_uuid === notif.collection_uuid) {
                notif.user_read = true;
              }
            }

            return notif;
          }),
        );
      }
    });
  };

  const markAllAsRead = () => {
    void fetch(`${apiHost}/notifications_readall`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        setNotifs(
          (prev) =>
            prev.map((notif) => {
              notif.user_read = true;
              return notif;
            }) as unknown as NotificationDTO[],
        );
      }
    });
  };

  return (
    <Show when={!!props.user}>
      <div>
        <Popover defaultOpen={false} class="relative flex items-center">
          {({ isOpen, setState }) => (
            <>
              <PopoverButton
                aria-label="Toggle user actions menu"
                classList={{ flex: true }}
                onClick={() => {
                  setPage(1);
                  fetchNotifs();
                }}
              >
                <IoNotificationsOutline class="mr-4 h-6 w-6 fill-current" />
                {notifs().find((notif) => !notif.user_read) && (
                  <span class="relative">
                    <div class="absolute right-3 top-0 h-2 w-2 rounded-full bg-red-500" />
                  </span>
                )}
              </PopoverButton>
              <Show when={isOpen() || usingPanel()}>
                <div>
                  <PopoverPanel
                    unmount={true}
                    class="absolute left-1/2 z-10 mt-5 h-fit w-fit -translate-x-[100%] transform rounded-md bg-neutral-100 p-1 px-0.5 dark:bg-neutral-700 dark:text-white sm:px-1"
                    onMouseEnter={() => setUsingPanel(true)}
                    onMouseLeave={() => setUsingPanel(false)}
                    onClick={() => setState(true)}
                  >
                    <Menu class="h-0">
                      <MenuItem class="h-0" as="button" aria-label="Empty" />
                    </Menu>
                    <div class="w-full  min-w-[200px] md:min-w-[300px]">
                      <div class="mb-1 flex items-center justify-center text-center align-middle text-sm font-semibold">
                        <div class="items-center text-center">
                          {"Notifications " +
                            (notifs().length > 0
                              ? `(${
                                  notifs().filter((notif) => !notif.user_read)
                                    .length
                                } pending)`
                              : "")}
                        </div>
                        <button
                          class="absolute right-2 flex justify-end"
                          onClick={() => markAllAsRead()}
                        >
                          <VsCheckAll class="h-4 w-4" />
                        </button>
                      </div>

                      <div class="scrollbar-track-rounded-md scrollbar-thumb-rounded-md flex max-h-[40vh] w-full transform flex-col space-y-1 overflow-hidden overflow-y-auto rounded-lg bg-neutral-200 px-0.5 shadow-2xl scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-600 dark:scrollbar-thumb-neutral-500 sm:px-1">
                        <For each={notifs()}>
                          {(notification) => {
                            return (
                              <SingleNotification
                                notification={notification}
                                notifs={notifs}
                                markAsRead={markAsRead}
                                setState={setState}
                                similarityScoreThreshold={
                                  similarityScoreThreshold
                                }
                              />
                            );
                          }}
                        </For>
                        <div class="flex items-center justify-between">
                          <div />
                          <div class="flex items-center">
                            <div class="text-sm text-neutral-400">
                              {page()} / {totalPages()}
                            </div>
                            <button
                              class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                              disabled={page() == 1}
                              onClick={() => {
                                setState(true);
                                setPage((prev) => prev - 1);
                                fetchNotifs();
                              }}
                            >
                              <BiRegularChevronLeft class="h-6 w-6 fill-current" />
                            </button>
                            <button
                              class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                              disabled={page() == totalPages()}
                              onClick={() => {
                                setState(true);
                                setPage((prev) => prev + 1);
                                fetchNotifs();
                              }}
                            >
                              <BiRegularChevronRight class="h-6 w-6 fill-current" />
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </PopoverPanel>
                </div>
              </Show>
            </>
          )}
        </Popover>
      </div>
    </Show>
  );
};
