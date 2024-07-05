pub mod cli;
pub mod clocks;
pub mod filesystem;
pub mod http;
pub mod io;
pub mod random;
pub mod sockets;

pub(crate) enum Error {
  /// No error occurred. System call completed successfully.
  Success = 0,
  /// Argument list too long.
  TooBig = 1,
  /// Permission denied.
  Acces = 2,
  /// Address in use.
  AddrInUse = 3,
  /// Address not available.
  AddrNotAvailable = 4,
  /// Address family not supported.
  AfNoSupport = 5,
  /// Resource unavailable, or operation would block.
  Again = 6,
  /// Connection already in progress.
  Already = 7,
  /// Bad file descriptor.
  Badf = 8,
  /// Bad message.
  BadMsg = 9,
  /// Device or resource busy.
  Busy = 10,
  /// Operation canceled.
  Canceled = 11,
  /// No child processes.
  Child = 12,
  /// Connection aborted.
  ConnAborted = 13,
  /// Connection refused.
  ConnRefused = 14,
  /// Connection reset.
  ConnReset = 15,
  /// Resource deadlock would occur.
  Deadlk = 16,
  /// Destination address required.
  DestAddrReq = 17,
  /// Mathematics argument out of domain of function.
  Dom = 18,
  /// Reserved.
  DQuot = 19,
  /// File exists.
  Exist = 20,
  /// Bad address.
  Fault = 21,
  /// File too large.
  FBig = 22,
  /// Host is unreachable.
  HostUnreach = 23,
  /// Identifier removed.
  IdRm = 24,
  /// Illegal byte sequence.
  IlSeq = 25,
  /// Operation in progress.
  InProgress = 26,
  /// Interrupted function.
  Intr = 27,
  /// Invalid argument.
  InVal = 28,
  /// I/O error.
  Io = 29,
  /// Socket is connected.
  IsConn = 30,
  /// Is a directory.
  IsDir = 31,
  /// Too many levels of symbolic links.
  Loop = 32,
  /// File descriptor value too large.
  MFile = 33,
  /// Too many links.
  MLink = 34,
  /// Message too large.
  MsgSize = 35,
  /// Reserved.
  MultiHop = 36,
  /// Filename too long.
  NameTooLong = 37,
  /// Network is down.
  NetDown = 38,
  /// Connection aborted by network.
  NetReset = 39,
  /// Network unreachable.
  NetUnreach = 40,
  /// Too many files open in system.
  NFile = 41,
  /// No buffer space available.
  NoBufs = 42,
  /// No such device.
  NoDev = 43,
  /// No such file or directory.
  NoEnt = 44,
  /// Executable file format error.
  NoExec = 45,
  /// No locks available.
  NoLck = 46,
  /// Reserved.
  NoLink = 47,
  /// Not enough space.
  NoMem = 48,
  /// No message of the desired type.
  NoMsg = 49,
  /// Protocol not available.
  NoProtoOpt = 50,
  /// No space left on device.
  NoSpc = 51,
  /// Function not supported.
  NoSys = 52,
  /// The socket is not connected.
  NotConn = 53,
  /// Not a directory or a symbolic link to a directory.
  NotDir = 54,
  /// Directory not empty.
  NotEmpty = 55,
  /// State not recoverable.
  NotRecoverable = 56,
  /// Not a socket.
  NotSock = 57,
  /// Not supported, or operation not supported on socket.
  NotSup = 58,
  /// Inappropriate I/O control operation.
  NotTy = 59,
  /// No such device or address.
  NxIo = 60,
  /// Value too large to be stored in data type.
  Overflow = 61,
  /// Previous owner died.
  OwnerDead = 62,
  /// Operation not permitted.
  Perm = 63,
  /// Broken pipe.
  Pipe = 64,
  /// Protocol error.
  Proto = 65,
  /// Protocol not supported.
  ProtoNoSupport = 66,
  /// Protocol wrong type for socket.
  Prototype = 67,
  /// Result too large.
  Range = 68,
  /// Read-only file system.
  Rofs = 69,
  /// Invalid seek.
  SpIpe = 70,
  /// No such process.
  Srch = 71,
  /// Reserved.
  Stale = 72,
  /// Connection timed out.
  TimedOut = 73,
  /// Text file busy.
  TxtBsy = 74,
  /// Cross-device link.
  XDev = 75,
  /// Extension: Capabilities insufficient.
  NotCapable = 76,
}