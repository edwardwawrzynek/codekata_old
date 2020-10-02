export const API_ROUTE = process.env.NODE_ENV === "development" ? "http://localhost:8000/api" : "/api";
export const GET_USER = `${API_ROUTE}/user`;
export const NEW_USER = `${API_ROUTE}/user/new`;
export const NEW_SESSION = `${API_ROUTE}/session/new`;
export const DELETE_SESSION = `${API_ROUTE}/session/delete`;
export const GEN_API_KEY = `${API_ROUTE}/user/generate_api`;
export const USER_EDIT = `${API_ROUTE}/user/edit`;
export const GAME_INDEX = `${API_ROUTE}/game/index`;
export const GAME_NEW = `${API_ROUTE}/game/new`;
export function GET_GAME(id: number) {
  return `${API_ROUTE}/game/${id}`;
}
export function GAME_JOIN(id: number) {
  return `${API_ROUTE}/game/${id}/join`;
}
export function GAME_LEAVE(id: number) {
  return `${API_ROUTE}/game/${id}/leave`;
}
export function GAME_START(id: number) {
  return `${API_ROUTE}/game/${id}/start`;
}
export function GAME_MOVE(id: number) {
  return `${API_ROUTE}/game/${id}/move`;
}

export const rejectedPromiseHandler = (e: any) => {
  alert(e);
}

export const checkError = (e: any): boolean => {
  if(e.error !== undefined) {
    alert(`error: ${e.error}`);
    return false;
  } else {
    return true;
  }
}

export interface SessionInfo {
  logged_in: boolean,
  has_api_key: boolean,
  username: string,
  display_name: string,
  id: number,
}

export function postArgs(args: {[key: string]: string}): URLSearchParams {
  const data = new URLSearchParams();
  for(const [key, value] of Object.entries(args)) {
    data.append(key, value);
  }

  return data;
}
