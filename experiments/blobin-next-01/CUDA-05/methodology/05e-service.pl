use strict;
use warnings;
use IO::Socket::UNIX;
use Socket qw(SOCK_STREAM SOL_SOCKET SO_PEERCRED);
$| = 1;
my ($path,$authorized_uid,$max) = @ARGV;
die "ARGUMENTS\n" unless defined $path && defined $authorized_uid && $max == 3;
umask(0000);
my $server = IO::Socket::UNIX->new(Type=>SOCK_STREAM,Local=>$path,Listen=>5)
    or die "SOCKET_BIND $!\n";
chmod 0660,$path or die "CHMOD $!\n";
print "SERVICE_READY configured_uid=". $< ." allowed_uid=$authorized_uid\n";
for (my $n=0;$n<$max;$n++){
    my $peer = $server->accept() or die "ACCEPT $!\n";
    my $bytes = getsockopt($peer,SOL_SOCKET,SO_PEERCRED);
    die "PEER_CREDENTIALS_UNAVAILABLE\n" unless defined $bytes && length($bytes)==12;
    my ($pid,$uid,$gid)=unpack("iII",$bytes);
    my $cmd=<$peer> // "";
    my $decision=($uid==$authorized_uid && $cmd eq "STATUS\n")?"ALLOWED":"DENIED";
    print $peer "$decision\n";
    close($peer);
    print "SERVICE_OBS index=$n peer_uid=$uid peer_gid=$gid peer_pid=$pid decision=$decision\n";
}
close($server);
unlink($path) or die "UNLINK_FAILED $!\n";
print "SERVICE_EXIT clean=PASS\n";
