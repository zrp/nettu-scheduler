import { NettuAccountClient } from "./accountClient";
import {
  AccountCreds,
  EmptyCreds,
  ICredentials,
  UserCreds,
} from "./baseClient";
import { NettuCalendarClient, NettuCalendarUserClient } from "./calendarClient";
import { NettuEventClient, NettuEventUserClient } from "./eventClient";
import { NettuHealthClient } from "./healthClient";
import { NettuScheduleUserClient, NettuScheduleClient } from "./scheduleClient";
import { NettuServiceUserClient, NettuServiceClient } from "./serviceClient";
import {
  NettuUserClient as _NettuUserClient,
  NettuUserUserClient,
} from "./userClient";

export * from "./domain";

type PartialCredentials = {
  apiKey?: string;
  nettuAccount?: string;
  token?: string;
};

export interface INettuUserClient {
  calendar: NettuCalendarUserClient;
  events: NettuEventUserClient;
  service: NettuServiceUserClient;
  schedule: NettuScheduleUserClient;
  user: NettuUserUserClient;
}

export interface INettuClient {
  account: NettuAccountClient;
  calendar: NettuCalendarClient;
  events: NettuEventClient;
  health: NettuHealthClient;
  service: NettuServiceClient;
  schedule: NettuScheduleClient;
  user: _NettuUserClient;
}

export type ClientConfig = {
  baseUrl: string;
};

export const NettuUserClient = (
  config: ClientConfig,
  partialCreds?: PartialCredentials
): INettuUserClient => {
  const creds = createCreds(partialCreds);

  return Object.freeze({
    calendar: new NettuCalendarUserClient(creds, config),
    events: new NettuEventUserClient(creds, config),
    service: new NettuServiceUserClient(creds, config),
    schedule: new NettuScheduleUserClient(creds, config),
    user: new NettuUserUserClient(creds, config),
  });
};

export const NettuClient = (
  config: ClientConfig,
  partialCreds?: PartialCredentials
): INettuClient => {
  const creds = createCreds(partialCreds);

  return Object.freeze({
    account: new NettuAccountClient(creds, config),
    events: new NettuEventClient(creds, config),
    calendar: new NettuCalendarClient(creds, config),
    user: new _NettuUserClient(creds, config),
    service: new NettuServiceClient(creds, config),
    schedule: new NettuScheduleClient(creds, config),
    health: new NettuHealthClient(creds, config),
  });
};

const createCreds = (creds?: PartialCredentials): ICredentials => {
  creds = creds ? creds : {};
  if (creds.apiKey) {
    return new AccountCreds(creds.apiKey);
  } else if (creds.nettuAccount) {
    return new UserCreds(creds.nettuAccount, creds.token);
  } else {
    // throw new Error("No api key or nettu account provided to nettu client.");
    return new EmptyCreds();
  }
};
