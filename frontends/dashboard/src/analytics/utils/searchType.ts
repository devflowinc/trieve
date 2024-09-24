export const formatSearchMethod = (searchMethod: string) => {
  switch (searchMethod) {
    case "hybrid":
      return "Hybrid";
    case "fulltext":
      return "Fulltext";
    case "semantic":
      return "Semantic";
    case "autocomplete":
      return "Autocomplete";
    default:
      return "All";
  }
};
