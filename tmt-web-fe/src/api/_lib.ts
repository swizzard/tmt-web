export const API_URL = new URL("http://0.0.0.0:8080/");
export function paginationDataToQuery(data?: {
  page?: number;
  pageSize?: number;
}): string | undefined {
  if (!data) return undefined;
  let pp: Record<string, string> = {};
  let anyData = false;
  if (data.page) {
    pp.page = data.page.toString();
    anyData = true;
  }
  if (data.pageSize) {
    pp.page_size = data.pageSize.toString();
    anyData = true;
  }
  if (!anyData) {
    return undefined;
  }
  return new URLSearchParams(pp).toString();
}
