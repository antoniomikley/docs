#!/bin/bash

while :
do
  t1=`date +%s%N`
  rm -f /tmp/firecracker.socket
  firecracker --api-sock /tmp/firecracker.socket --config-file vm_config.json &
  wait
  t2=`date +%s%N`
  echo "scale=3 ; ($t2-$t1) / 1000000000" | bc >> fc_times
done


