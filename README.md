fuseqemu
---

Fork of https://github.com/vi/fusenbd

A FUSE mounter for QEMU block device.

Example
---

```
qemu-img create -f qcow2 image.qcow

./fuseqemu image.qcow image.dd &

while ! test -s image.dd; do sleep 1; done

mkfs.ext4 image.dd

mkdir image.mnt

./lklfuse image.dd image.mnt -o type=ext4

ls -la image.mnt

fusermount -u image.mnt

fusermount -u image.dd
```
