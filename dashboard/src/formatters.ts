export const usdFormatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
});

export const formatBytesDecimal = (bytes: number, dm = 2) => {
  if (bytes == 0) return "0 Bytes";
  if (bytes > 1e8) {
    return (bytes / 1e9).toFixed(dm) + " GB";
  }
  return (bytes / 1e6).toFixed(dm) + " MB";
};

export const numberFormatter = new Intl.NumberFormat("en-US");

export const formatDate = (date: Date) =>
  new Intl.DateTimeFormat("en-US").format(date);
