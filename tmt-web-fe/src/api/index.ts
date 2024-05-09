export {
  authorize,
  logout,
  signup,
  confirmUser,
  renewToken,
  type ConfirmUserInput,
  type LoginInput,
  type SignupInput,
} from "./users";
export {
  createTab,
  deleteTab,
  getTabDetails,
  getUserTabs,
  updateTab,
  type NewTabInput,
  type NewTabRequest,
  type Tab,
  type Tag,
  type TabWithTags,
  type UpdateTabInput,
  type UpdateTabRequest,
  type UserListTab,
  type UserTabsResult,
} from "./tabs";
export interface AuthorizedPost<T> {
  authToken: string;
  data: T;
}

export interface AuthorizedGet {
  authToken: string;
}

export interface AuthorizedDelete {
  authToken: string;
}

export interface PaginatedAuthorizedGet {
  authToken: string;
  page?: number;
  pageSize?: number;
}

export class TMTError extends Error {
  isToken: boolean;
  constructor(message: string) {
    super(message);
    this.isToken = TMTError.isTokenError(message);
  }
  static isTokenError(message: string): boolean {
    return message === "Invalid token";
  }
}
