export type RawHandle = {
  id: bigint;
};

export type Syscall =
  | { Sleep: number }
  | { IpcCreate: string }
  | { IpcConnect: string }
  | { Send: [RawHandle, string] };

export type SyscallError =
  | "NoSuchEntry"
  | "AlreadyExists"
  | "UnknownHandle"
  | "NotAllowedHandle"
  | "NotImplemented"
  | "UnreachableEntry";

export type SyscallData =
  | { Fail: SyscallError }
  | { Handle: RawHandle }
  | { Connection: { client: RawHandle; server: RawHandle } }
  | { ReceivingData: { focus: RawHandle; data: string } }
  | "None";

export type PollResult = "Pending" | { Syscall: Syscall } | { Done: bigint };