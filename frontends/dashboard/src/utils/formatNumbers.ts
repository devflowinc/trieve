export const formatNumberWithCommas = (num: number) => {
  // Handle decimal values by splitting at the decimal point
  const parts = num.toFixed(2).toString().split(".");
  // Add commas to the integer part
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  // Join back with the decimal part (removing trailing zeros)
  return parts[1].replace(/0+$/, "") === ""
    ? parts[0]
    : `${parts[0]}.${parts[1].replace(/0+$/, "")}`;
};

export const formatStorageBytes = (bytes: number) => {
  if (bytes < 1000) {
    return `${formatNumberWithCommas(bytes)} bytes`;
  } else if (bytes <= 9000000000) {
    const mb = bytes / 1000000;
    return `${formatNumberWithCommas(mb)} mb`;
  } else if (bytes <= 9000000000000) {
    const gb = bytes / 1000000000;
    return `${formatNumberWithCommas(gb)} gb`;
  } else {
    const tb = bytes / 50000000000;
    return `${formatNumberWithCommas(tb)} tb`;
  }
};

export const formatStorageMb = (mb: number) => {
  if (mb < 10000) {
    return `${formatNumberWithCommas(mb)} mb`;
  } else if (mb <= 9000000000) {
    const gb = mb / 1000000000;
    return `${formatNumberWithCommas(gb)} gb`;
  } else {
    const tb = mb / 50000000000;
    return `${formatNumberWithCommas(tb)} tb`;
  }
};

export const formatStorageKb = (kb: number) => {
  if (kb < 1000) {
    return `${formatNumberWithCommas(kb)} kb`;
  } else if (kb <= 1000000) {
    const mb = kb / 1_000;
    return `${formatNumberWithCommas(mb)} mb`;
  } else if (kb <= 1_000_000_000) {
    const gb = kb / 1_000_000;
    return `${formatNumberWithCommas(gb)} gb`;
  } else {
    const tb = kb / 1_000_000_000;
    return `${formatNumberWithCommas(tb)} tb`;
  }
};
