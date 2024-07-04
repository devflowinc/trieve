export function formatNumberWithCommas(num: number) {
  return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

export function formatStorage(mb: number) {
  console.log(mb);
  if (mb < 1000) {
    return `${formatNumberWithCommas(mb)} mb`;
  } else if (mb < 1000000) {
    const gb = mb / 1024;
    return `${formatNumberWithCommas(gb)} gb`;
  } else {
    const tb = mb / (1024 * 1024);
    return `${formatNumberWithCommas(tb)} tb`;
  }
}
