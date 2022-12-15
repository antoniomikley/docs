#!/bin/bash
while :
do
  t1=`date +%s%N`
  ~/cloud-hypervisor/cloud-hypervisor/target/release/cloud-hypervisor \
    --kernel vmlinux.bin \
	  --disk path=rootfs.ext4 \
    --cmdline "console=hvc0 root=/dev/vda rw" \
	  --cpus boot=2 \
	  --memory size=1024M
  wait
  t2=`date +%s%N`
  echo "scale=3; ($t2-$t1) / 1000000000" | bc >> ch_times
done
