fuseqemu
---

Fork of https://github.com/vi/fusenbd

A FUSE mounter for QEMU block device.

Example
---

```
$ fuseqemu data image.qcow -r -- -o auto_unmount,default_permissions,allow_other,ro&
[1] 14013

$ mkdir -p m

$ ntfs-3g -o ro ./data m

$ ls m
Boot  bootmgr  BOOTSECT.BAK  System Volume Information

$ fusermount -u m

$ fusermount -u data
[1]+  Done         fuseqemu 
```
