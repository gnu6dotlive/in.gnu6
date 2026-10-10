#!/usr/bin/env python3
"""Isolated recovery test; never operates on the workstation's real user socket."""
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile

SCRIPT=Path(__file__).with_name('recover_stale_socket.py')
with tempfile.TemporaryDirectory(prefix='blobin-recover-test-') as root:
    environ=dict(os.environ,XDG_RUNTIME_DIR=root)
    def check():return subprocess.run([sys.executable,str(SCRIPT)],env=environ,text=True,capture_output=True,timeout=3)
    assert check().returncode==0,'fresh runtime folder'
    target=Path(root)/'blobin-showcase/render.sock'
    with socket.socket(socket.AF_UNIX,socket.SOCK_SEQPACKET) as live:
        live.bind(str(target));live.listen(1)
        assert check().returncode!=0 and target.exists(),'active listener must be preserved'
    assert check().returncode==0 and not target.exists(),'stale socket should be removed'
    target.write_text('NOT A SOCKET')
    assert check().returncode!=0 and target.is_file(),'regular file must be preserved'
    target.unlink()
    folder=target.parent
    folder.chmod(0o755)
    assert check().returncode!=0,'shared directory denied'
    folder.chmod(0o700)
print('BLOB_STALE_SOCKET_RECOVERY_PASS cases=5 active_listener=preserved stale=removed unknown_file=preserved')
