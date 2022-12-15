#!/bin/bash
while :
do
  t1=`date +%s%N`
  ~/crosvm/target/debug/crosvm run --rwdisk "rootfs.ext4" -p "root=/dev/vda" vmlinux.bin
  wait
  t2=`date +%s%N`
  echo "scale=3; ($t2-$t1) / 1000000000" | bc >> crosvmtimes
done
