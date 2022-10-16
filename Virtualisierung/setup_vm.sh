#!/bin/bash

vm_name=$1
ram=$2
cores=$3
size=$4
path=$5

exec virt-install \
  --name $vm_name \
  --memory $ram \
  --vcpus=$cores \
  --disk size=$size,format=qcow2 \
  --os-variant detect=on,require=on \
  --cpu host \
  --cdrom $path \
  --network default \
  --virt-type kvm \
  --graphics spice
