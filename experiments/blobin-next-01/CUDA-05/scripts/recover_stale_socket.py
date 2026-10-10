#!/usr/bin/env python3
"""Safely recover a dead same-UID seqpacket listener; never replace an active one."""
import errno
import os
from pathlib import Path
import socket
import stat
import sys

runtime=os.environ.get('XDG_RUNTIME_DIR')
if not runtime or not Path(runtime).is_dir():sys.exit('BLOB_RECOVERY_NO_USER_RUNTIME_DIR')
folder=Path(runtime)/'blobin-showcase'
if folder.is_symlink():sys.exit('BLOB_RECOVERY_REFUSE_SYMLINK_DIRECTORY')
folder.mkdir(mode=0o700,exist_ok=True)
fm=folder.lstat()
if not stat.S_ISDIR(fm.st_mode) or fm.st_uid!=os.geteuid() or (fm.st_mode & 0o077):
    sys.exit('BLOB_RECOVERY_REFUSE_UNOWNED_OR_SHARED_DIRECTORY')
p=folder/'render.sock'
try:found=p.lstat()
except FileNotFoundError:
    print('BLOB_RECOVERY_READY no_previous_socket')
    sys.exit(0)
if not stat.S_ISSOCK(found.st_mode) or found.st_uid!=os.geteuid():
    sys.exit('BLOB_RECOVERY_REFUSE_UNKNOWN_NODE')
with socket.socket(socket.AF_UNIX,socket.SOCK_SEQPACKET) as probe:
    probe.settimeout(1.0)
    try:probe.connect(str(p))
    except OSError as exc:
        if exc.errno not in (errno.ECONNREFUSED,errno.ENOENT):
            sys.exit('BLOB_RECOVERY_REFUSE_UNCERTAIN_SOCKET '+str(exc.errno))
    else:
        sys.exit('BLOB_RECOVERY_REFUSE_ACTIVE_LISTENER')
try:now=p.lstat()
except FileNotFoundError:sys.exit(0)
if now.st_dev!=found.st_dev or now.st_ino!=found.st_ino or not stat.S_ISSOCK(now.st_mode):
    sys.exit('BLOB_RECOVERY_REFUSE_SOCKET_CHANGED')
p.unlink()
print('BLOB_RECOVERY_STALE_SOCKET_REMOVED owner_verified=YES')
