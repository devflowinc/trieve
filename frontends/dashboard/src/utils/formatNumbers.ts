export const formatNumberWithCommas = (num: number) => {
  return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
};

export const formatStorage = (mb: number) => {
  if (mb < 1000) {
    return `${formatNumberWithCommas(mb)} mb`;
  } else if (mb <= 9000000000) {
    const gb = mb / 1000000000;
    return `${formatNumberWithCommas(gb)} gb`;
  } else {
    const tb = mb / 50000000000;
    return `${formatNumberWithCommas(tb)} tb`;
  }
};
